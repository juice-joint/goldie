extern crate ffmpeg_next as ffmpeg;

use std::env;
use std::path::Path;
use ffmpeg::{codec, filter, format, frame, media, Rescale};

fn main() {
    ffmpeg::init().unwrap();

    let input = env::args().nth(1).expect("missing input");
    let output = env::args().nth(2).expect("missing output");

    // Open input and output formats
    let mut ictx = format::input(&input).unwrap();
    let mut octx = format::output(&output).unwrap();

    // Find best audio and video streams
    let audio_stream = ictx
        .streams()
        .best(media::Type::Audio)
        .expect("No audio stream found");
    let video_stream = ictx
        .streams()
        .best(media::Type::Video)
        .expect("No video stream found");

    let audio_index = audio_stream.index();
    let video_index = video_stream.index();

    // Set up audio filter: rubberband=pitch=0.9438743126816935
    let mut decoder = audio_stream.codec().decoder().audio().unwrap();
    let mut encoder = ffmpeg::encoder::find(codec::Id::AAC).unwrap().audio().unwrap();
    encoder.set_rate(decoder.rate() as i32);
    encoder.set_channels(decoder.channels());
    encoder.set_format(codec::Sample::FLTP);

    let mut filter_graph = filter::Graph::new();
    filter_graph
        .add(&filter::find("abuffer").unwrap(), "in", &format!(
            "time_base={}:sample_rate={}:sample_fmt={}:channel_layout=0x{:x}",
            decoder.time_base(),
            decoder.rate(),
            decoder.format().name(),
            decoder.channel_layout().bits()
        ))
        .unwrap();
    filter_graph
        .add(&filter::find("rubberband").unwrap(), "rubberband", "pitch=0.9438743126816935")
        .unwrap();
    filter_graph
        .add(&filter::find("abuffersink").unwrap(), "out", "")
        .unwrap();
    filter_graph.output("rubberband", 0).unwrap().input("out", 0).unwrap().parse("").unwrap();
    filter_graph.validate().unwrap();

    // Add streams to output
    let mut audio_out = octx.add_stream(encoder).unwrap();
    let mut video_out = octx.add_stream(video_stream.codec()).unwrap();

    audio_out.set_time_base((1, decoder.rate() as i32));
    video_out.set_time_base(video_stream.time_base());

    octx.set_metadata(ictx.metadata().to_owned());
    octx.write_header().unwrap();

    // Process packets
    for (stream, mut packet) in ictx.packets() {
        if stream.index() == audio_index {
            decoder.send_packet(&packet).unwrap();
            let mut decoded = frame::Audio::empty();
            while decoder.receive_frame(&mut decoded).is_ok() {
                filter_graph.get("in").unwrap().source().add(&decoded).unwrap();
                let mut filtered = frame::Audio::empty();
                while filter_graph.get("out").unwrap().sink().frame(&mut filtered).is_ok() {
                    encoder.send_frame(&filtered).unwrap();
                    let mut encoded = ffmpeg::Packet::empty();
                    while encoder.receive_packet(&mut encoded).is_ok() {
                        encoded.write_interleaved(&mut octx).unwrap();
                    }
                }
            }
        } else if stream.index() == video_index {
            // Copy video packets directly
            packet.set_stream(video_out.index());
            packet.write_interleaved(&mut octx).unwrap();
        }
    }

    octx.write_trailer().unwrap();
}