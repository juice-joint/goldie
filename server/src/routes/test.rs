extern crate ffmpeg_next as ffmpeg;

use std::collections::HashMap;
use std::env;
use std::path::Path;

use ffmpeg::{codec, encoder, filter, format, frame, media, Rational, Stream};

struct AudioTranscoder {
    stream_index: usize,
    filter: filter::Graph,
    in_time_base: ffmpeg::Rational,
    out_time_base: ffmpeg::Rational,
    decoder: codec::decoder::Audio,
    encoder: codec::encoder::Audio,
}

impl AudioTranscoder {
    fn new<P: AsRef<Path> + ?Sized>(
        audio_stream: &Stream,
        octx: &mut format::context::Output,
        path: &P,
        filter_spec: &str,
    ) -> Result<Self, ffmpeg::Error> {
        // Initialize decoder with error handling
        let decoder = {
            let decoder_context =
                ffmpeg::codec::context::Context::from_parameters(audio_stream.parameters())?;
            let mut decoder = decoder_context.decoder().audio()?;
            decoder.set_parameters(audio_stream.parameters())?;
            decoder
        };

        // Find and validate audio codec
        let format_flags = octx.format().flags();
        let codec = ffmpeg::encoder::find(octx.format().codec(path, media::Type::Audio))
            .ok_or(ffmpeg::Error::EncoderNotFound)?
            .audio()?;

        // Create output stream
        let mut output = octx.add_stream(codec)?;

        // Configure encoder
        let encoder = {
            let context = ffmpeg::codec::context::Context::from_parameters(output.parameters())?;
            let mut encoder = context.encoder().audio()?;

            // Determine channel layout with fallback
            let channel_layout = codec
                .channel_layouts()
                .map(|cls| cls.best(decoder.channel_layout().channels()))
                .unwrap_or(ffmpeg::channel_layout::ChannelLayout::STEREO);

            // Configure encoder parameters
            encoder.set_rate(decoder.rate() as i32);
            encoder.set_channel_layout(channel_layout);

            encoder.set_format(
                codec
                    .formats()
                    .expect("unknown supported formats")
                    .next()
                    .unwrap(),
            );

            // Set bitrates if available
            encoder.set_bit_rate(decoder.bit_rate());
            encoder.set_max_bit_rate(decoder.max_bit_rate());

            encoder.set_time_base((1, decoder.rate() as i32));

            if format_flags.contains(ffmpeg::format::flag::Flags::GLOBAL_HEADER) {
                encoder.set_flags(ffmpeg::codec::flag::Flags::GLOBAL_HEADER);
            }

            encoder
        };

        // Set output timebase
        let time_base = (1, decoder.rate() as i32);
        output.set_time_base(time_base);

        // Open and configure final encoder
        let encoder = encoder.open_as(codec)?;
        output.set_parameters(&encoder);

        // Create filter
        let filter = filter(filter_spec, &decoder, &encoder)?;

        Ok(AudioTranscoder {
            stream_index: audio_stream.index(),
            filter,
            in_time_base: decoder.time_base(),
            out_time_base: output.time_base(),
            decoder,
            encoder,
        })
    }

    fn send_frame_to_encoder(&mut self, frame: &ffmpeg::Frame) {
        self.encoder.send_frame(frame).unwrap();
    }

    fn send_eof_to_encoder(&mut self) {
        self.encoder.send_eof().unwrap();
    }

    fn receive_and_process_encoded_packets(&mut self, octx: &mut format::context::Output) {
        let mut encoded = ffmpeg::Packet::empty();
        while self.encoder.receive_packet(&mut encoded).is_ok() {
            encoded.set_stream(self.stream_index);
            encoded.rescale_ts(self.in_time_base, self.out_time_base);
            encoded.write_interleaved(octx).unwrap();
        }
    }

    fn add_frame_to_filter(&mut self, frame: &ffmpeg::Frame) {
        self.filter.get("in").unwrap().source().add(frame).unwrap();
    }

    fn flush_filter(&mut self) {
        self.filter.get("in").unwrap().source().flush().unwrap();
    }

    fn get_and_process_filtered_frames(&mut self, octx: &mut format::context::Output) {
        let mut filtered = frame::Audio::empty();
        while self
            .filter
            .get("out")
            .unwrap()
            .sink()
            .frame(&mut filtered)
            .is_ok()
        {
            self.send_frame_to_encoder(&filtered);
            self.receive_and_process_encoded_packets(octx);
        }
    }

    fn send_packet_to_decoder(&mut self, packet: &ffmpeg::Packet) {
        self.decoder.send_packet(packet).unwrap();
    }

    fn send_eof_to_decoder(&mut self) {
        self.decoder.send_eof().unwrap();
    }

    fn receive_and_process_decoded_frames(&mut self, octx: &mut format::context::Output) {
        let mut decoded = frame::Audio::empty();
        while self.decoder.receive_frame(&mut decoded).is_ok() {
            let timestamp = decoded.timestamp();
            decoded.set_pts(timestamp);
            self.add_frame_to_filter(&decoded);
            self.get_and_process_filtered_frames(octx);
        }
    }
}

fn filter(
    spec: &str,
    decoder: &codec::decoder::Audio,
    encoder: &codec::encoder::Audio,
) -> Result<filter::Graph, ffmpeg::Error> {
    let mut filter = filter::Graph::new();

    let args = format!(
        "time_base={}:sample_rate={}:sample_fmt={}:channel_layout=0x{:x}",
        decoder.time_base(),
        decoder.rate(),
        decoder.format().name(),
        decoder.channel_layout().bits()
    );

    filter.add(&filter::find("abuffer").unwrap(), "in", &args)?;
    filter.add(&filter::find("abuffersink").unwrap(), "out", "")?;

    {
        let mut out = filter.get("out").unwrap();

        out.set_sample_format(encoder.format());
        out.set_channel_layout(encoder.channel_layout());
        out.set_sample_rate(encoder.rate());
    }

    filter.output("in", 0)?.input("out", 0)?.parse(spec)?;
    filter.validate()?;

    if let Some(codec) = encoder.codec() {
        if !codec
            .capabilities()
            .contains(ffmpeg::codec::capabilities::Capabilities::VARIABLE_FRAME_SIZE)
        {
            filter
                .get("out")
                .unwrap()
                .sink()
                .set_frame_size(encoder.frame_size());
        }
    }

    Ok(filter)
}

fn main() {
    ffmpeg::init().unwrap();

    let input = env::args().nth(1).expect("missing input");
    let output = env::args().nth(2).expect("missing output");
    let filter = env::args().nth(3).unwrap_or_else(|| "anull".to_owned());

    let mut ictx = format::input(&input).unwrap();
    let mut octx = format::output(&output).unwrap();

    let mut audio_transcoders = HashMap::new();
    let mut stream_mapping: Vec<isize> = vec![0; ictx.nb_streams() as _];

    let mut ist_time_bases = vec![Rational(0, 0); ictx.nb_streams() as _];
    let mut ost_time_bases = vec![Rational(0, 0); ictx.nb_streams() as _];
    let mut output_stream_index = 0;

    for (ist_index, ist) in ictx.streams().enumerate() {
        let ist_medium = ist.parameters().medium();
        if ist_medium != media::Type::Audio
            && ist_medium != media::Type::Video
            && ist_medium != media::Type::Subtitle
        {
            stream_mapping[ist_index] = -1;
            continue;
        }

        stream_mapping[ist_index] = output_stream_index;
        ist_time_bases[ist_index] = ist.time_base();

        if ist_medium == media::Type::Audio {
            // Initialize transcoder for video stream.
            audio_transcoders.insert(
                ist_index,
                AudioTranscoder::new(&ist, &mut octx, &output, &filter).unwrap(),
            );
        } else {
            // Set up for stream copy for non-video stream.
            let mut ost = octx.add_stream(encoder::find(codec::Id::None)).unwrap();
            ost.set_parameters(ist.parameters());
            // We need to set codec_tag to 0 lest we run into incompatible codec tag
            // issues when muxing into a different container format. Unfortunately
            // there's no high level API to do this (yet).
            unsafe {
                (*ost.parameters().as_mut_ptr()).codec_tag = 0;
            }
        }
        output_stream_index += 1;
    }

    octx.set_metadata(ictx.metadata().to_owned());
    octx.write_header().unwrap();

    for (ost_index, _) in octx.streams().enumerate() {
        ost_time_bases[ost_index] = octx.stream(ost_index as _).unwrap().time_base();
    }

    for (stream, mut packet) in ictx.packets() {
        let ist_index = stream.index();
        let output_stream_index = stream_mapping[ist_index];
        if output_stream_index == -1 {
            continue;
        }

        let ist_index: usize = stream.index();
        let ost_time_base = ost_time_bases[output_stream_index as usize];
        match audio_transcoders.get_mut(&ist_index) {
            Some(audio_transcoder) => {
                audio_transcoder.send_packet_to_decoder(&packet);
                audio_transcoder.receive_and_process_decoded_frames(&mut octx);
            }
            None => {
                // Do stream copy on non-audio streams.
                packet.rescale_ts(ist_time_bases[ist_index], ost_time_base);
                packet.set_position(-1);
                packet.set_stream(output_stream_index as _);
                packet.write_interleaved(&mut octx).unwrap();
            }
        }
    }

    for (_, audio_transcoder) in audio_transcoders.iter_mut() {
        audio_transcoder.send_eof_to_decoder();
        audio_transcoder.receive_and_process_decoded_frames(&mut octx);

        audio_transcoder.flush_filter();
        audio_transcoder.get_and_process_filtered_frames(&mut octx);

        audio_transcoder.send_eof_to_encoder();
        audio_transcoder.receive_and_process_encoded_packets(&mut octx);
    }

    octx.write_trailer().unwrap();
}
