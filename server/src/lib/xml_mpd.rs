use std::{fs, path::{Path, PathBuf}};

use regex::Regex;
use serde::{Deserialize, Serialize};
use quick_xml::{de::from_str, se::to_string};

fn is_empty_string(s: &str) -> bool {
    s.is_empty()
}

fn is_empty_option_string(s: &Option<String>) -> bool {
    matches!(s, None) || s.as_ref().map_or(true, |s| s.is_empty())
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename = "MPD")]
pub struct MPD {
    #[serde(rename = "@xmlns:xsi")]
    #[serde(default)]
    #[serde(skip_serializing_if = "is_empty_option_string")]
    pub xmlns_xsi: Option<String>,
    #[serde(rename = "@xmlns")]
    #[serde(default)]
    #[serde(skip_serializing_if = "is_empty_option_string")]
    pub xmlns: Option<String>,
    #[serde(rename = "@xmlns:xlink")]
    #[serde(default)]
    #[serde(skip_serializing_if = "is_empty_option_string")]
    pub xmlns_xlink: Option<String>,
    #[serde(rename = "@xsi:schemaLocation")]
    #[serde(default)]
    #[serde(skip_serializing_if = "is_empty_option_string")]
    pub schema_location: Option<String>,
    #[serde(rename = "@profiles")]
    #[serde(skip_serializing_if = "is_empty_string")]
    pub profiles: String,
    #[serde(rename = "@type")]
    #[serde(skip_serializing_if = "is_empty_string")]
    pub type_: String,
    #[serde(rename = "@mediaPresentationDuration")]
    #[serde(skip_serializing_if = "is_empty_string")]
    pub media_presentation_duration: String,
    #[serde(rename = "@maxSegmentDuration")]
    #[serde(skip_serializing_if = "is_empty_string")]
    pub max_segment_duration: String,
    #[serde(rename = "@minBufferTime")]
    #[serde(skip_serializing_if = "is_empty_string")]
    pub min_buffer_time: String,
    #[serde(rename = "ProgramInformation")]
    pub program_information: ProgramInformation,
    #[serde(rename = "ServiceDescription")]
    pub service_description: ServiceDescription,
    #[serde(rename = "Period")]
    pub period: Period,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProgramInformation {}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceDescription {
    #[serde(rename = "@id")]
    #[serde(skip_serializing_if = "is_empty_string")]
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Period {
    #[serde(rename = "@id")]
    #[serde(skip_serializing_if = "is_empty_string")]
    pub id: String,
    #[serde(rename = "@start")]
    #[serde(skip_serializing_if = "is_empty_string")]
    pub start: String,
    #[serde(rename = "AdaptationSet")]
    pub adaptation_sets: Vec<AdaptationSet>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdaptationSet {
    #[serde(rename = "@id")]
    #[serde(skip_serializing_if = "is_empty_string")]
    pub id: String,
    #[serde(rename = "@contentType")]
    #[serde(skip_serializing_if = "is_empty_string")]
    pub content_type: String,
    #[serde(rename = "@startWithSAP")]
    #[serde(skip_serializing_if = "is_empty_string")]
    pub start_with_sap: String,
    #[serde(rename = "@segmentAlignment")]
    #[serde(skip_serializing_if = "is_empty_string")]
    pub segment_alignment: String,
    #[serde(rename = "@bitstreamSwitching")]
    #[serde(skip_serializing_if = "is_empty_string")]
    pub bitstream_switching: String,
    #[serde(rename = "@frameRate")]
    #[serde(default)]
    #[serde(skip_serializing_if = "is_empty_option_string")]
    pub frame_rate: Option<String>,
    #[serde(rename = "@maxWidth")]
    #[serde(default)]
    #[serde(skip_serializing_if = "is_empty_option_string")]
    pub max_width: Option<String>,
    #[serde(rename = "@maxHeight")]
    #[serde(default)]
    #[serde(skip_serializing_if = "is_empty_option_string")]
    pub max_height: Option<String>,
    #[serde(rename = "@par")]
    #[serde(default)]
    #[serde(skip_serializing_if = "is_empty_option_string")]
    pub par: Option<String>,
    #[serde(rename = "@lang")]
    #[serde(default)]
    #[serde(skip_serializing_if = "is_empty_option_string")]
    pub lang: Option<String>,
    #[serde(rename = "Representation")]
    pub representation: Representation,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Representation {
    #[serde(rename = "@id")]
    #[serde(skip_serializing_if = "is_empty_string")]
    pub id: String,
    #[serde(rename = "@mimeType")]
    #[serde(skip_serializing_if = "is_empty_string")]
    pub mime_type: String,
    #[serde(rename = "@codecs")]
    #[serde(skip_serializing_if = "is_empty_string")]
    pub codecs: String,
    #[serde(rename = "@bandwidth")]
    #[serde(skip_serializing_if = "is_empty_string")]
    pub bandwidth: String,
    #[serde(rename = "@audioSamplingRate")]
    #[serde(default)]
    #[serde(skip_serializing_if = "is_empty_option_string")]
    pub audio_sampling_rate: Option<String>,
    #[serde(rename = "@width")]
    #[serde(default)]
    #[serde(skip_serializing_if = "is_empty_option_string")]
    pub width: Option<String>,
    #[serde(rename = "@height")]
    #[serde(default)]
    #[serde(skip_serializing_if = "is_empty_option_string")]
    pub height: Option<String>,
    #[serde(rename = "@scanType")]
    #[serde(default)]
    #[serde(skip_serializing_if = "is_empty_option_string")]
    pub scan_type: Option<String>,
    #[serde(rename = "@sar")]
    #[serde(default)]
    #[serde(skip_serializing_if = "is_empty_option_string")]
    pub sar: Option<String>,
    #[serde(rename = "AudioChannelConfiguration")]
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_channel_configuration: Option<AudioChannelConfiguration>,
    #[serde(rename = "SegmentTemplate")]
    pub segment_template: SegmentTemplate,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AudioChannelConfiguration {
    #[serde(rename = "@schemeIdUri")]
    #[serde(skip_serializing_if = "is_empty_string")]
    pub scheme_id_uri: String,
    #[serde(rename = "@value")]
    #[serde(skip_serializing_if = "is_empty_string")]
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SegmentTemplate {
    #[serde(rename = "@timescale")]
    #[serde(skip_serializing_if = "is_empty_string")]
    pub timescale: String,
    #[serde(rename = "@initialization")]
    #[serde(skip_serializing_if = "is_empty_string")]
    pub initialization: String,
    #[serde(rename = "@media")]
    #[serde(skip_serializing_if = "is_empty_string")]
    pub media: String,
    #[serde(rename = "@startNumber")]
    #[serde(skip_serializing_if = "is_empty_string")]
    pub start_number: String,
    #[serde(rename = "SegmentTimeline")]
    pub segment_timeline: SegmentTimeline,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SegmentTimeline {
    #[serde(rename = "S")]
    pub segments: Vec<Segment>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Segment {
    #[serde(rename = "@t")]
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub t: Option<String>,
    #[serde(rename = "@d")]
    #[serde(skip_serializing_if = "is_empty_string")]
    pub d: String,
    #[serde(rename = "@r")]
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r: Option<String>,
}

pub struct XmlMpdUtil;

impl XmlMpdUtil {
    pub fn find_mpd_files(base_dir: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
        let mut mpd_files = Vec::new();
        let output_dir = base_dir;
    
        if !output_dir.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Output directory not found: {}", output_dir.display()),
            ));
        }
    
        // Read all pitch directories
        for entry in fs::read_dir(&output_dir)? {
            let entry = entry?;
            let path = entry.path();
    
            if path.is_dir()
                && path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.starts_with("pitch") || n.starts_with("video"))
                    .unwrap_or(false)
            {
                let mpd_path = path.join("stream.mpd");
                if mpd_path.exists() {
                    mpd_files.push(mpd_path);
                }
            }
        }
    
        // Sort the files to ensure consistent processing order
        mpd_files.sort();
    
        if mpd_files.is_empty() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No MPD files found in pitch directories",
            ));
        }
    
        Ok(mpd_files)
    }
}