use std::fmt::Display;
use std::{fs::File, path::Path};

use actix::prelude::*;

use symphonia::core::codecs::FinalizeResult;
use symphonia::core::errors::{Error, Result};
use symphonia::core::formats::{SeekMode, SeekTo};
use symphonia::core::meta::{ColorMode, MetadataRevision, Tag, Value, Visual};
use symphonia::core::units::{Time, TimeBase};
use symphonia::core::{
    codecs::{DecoderOptions, CODEC_TYPE_NULL},
    formats::{FormatOptions, FormatReader, Track},
    io::MediaSourceStream,
    meta::MetadataOptions,
    probe::Hint,
};

use log::warn;

use crate::output;

const LOG_TARGET: &str = "player";

enum InternalPlayerCommands {
    Stop,
    Pause,
    Resume,
}

pub(crate) fn handle_request(
    receiver: std::sync::mpsc::Receiver<PlayerCommand>,
    sync_sender: std::sync::mpsc::Sender<PlayerUpdate>,
) {
    let mut current_sender: Option<std::sync::mpsc::Sender<InternalPlayerCommands>> = None;
    loop {
        if let Ok(command) = receiver.try_recv() {
            log::debug!(target: LOG_TARGET,"handling command: {:?}", &command);
            let sync_sender_clone = sync_sender.clone();

            match &command {
                PlayerCommand::Pause => {
                    if current_sender.is_some() {
                        log::debug!(target: LOG_TARGET,"pausing play");
                        _ = current_sender
                            .as_ref()
                            .unwrap()
                            .send(InternalPlayerCommands::Pause);
                    }
                }
                PlayerCommand::Resume => {
                    log::debug!(target: LOG_TARGET,"resuming play");
                    _ = current_sender
                        .as_ref()
                        .unwrap()
                        .send(InternalPlayerCommands::Resume);
                }
                PlayerCommand::Play(path) => {
                    if let Some(sender) = &current_sender {
                        _ = sender.send(InternalPlayerCommands::Stop);
                    }
                    let (sender, receiver) = std::sync::mpsc::channel::<InternalPlayerCommands>();
                    current_sender = Some(sender);

                    log::debug!(target: LOG_TARGET,"playing \"{:?}\"", &path);

                    // TODO: Make the abrupt stop easy to the ears. Example cross fade or something
                    let the_path = path.clone().to_string();
                    _ = std::thread::spawn(move || {
                        play_music(&the_path, receiver, sync_sender_clone);
                    });
                }
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

#[derive(Debug, Clone)]
pub(crate) enum PlayerCommand {
    Pause,
    Resume,
    Play(String),
}

#[derive(Message)]
#[rtype(result = "()")]
pub(crate) enum PlayerUpdate {
    Progress {
        position: (u64, u64, f64),
        total: (u64, u64, f64),
    },
}

impl Display for PlayerUpdate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Progress {
                position: (h, m, s),
                total: (th, tm, ts),
            } => {
                let total = format!("{{\"h\": {}, \"m\": {}, \"s\": {}}}", th, tm, ts);
                let pos = format!("{{\"h\": {}, \"m\": {}, \"s\": {}}}", h, m, s);
                write!(f, "{{\"pos\": {}, \"total\": {} }}", pos, total)
            }
        }
    }
}

fn play_music(
    path: &str,
    receiver: std::sync::mpsc::Receiver<InternalPlayerCommands>,
    sync_sender: std::sync::mpsc::Sender<PlayerUpdate>,
) {
    log::debug!(target: LOG_TARGET,"playing track: {}", path);
    let mut hint = Hint::new();
    let path = Path::new(path);

    if let Some(extension) = path.extension() {
        if let Some(extension_str) = extension.to_str() {
            hint.with_extension(extension_str);
        }
    }

    let source = Box::new(File::open(path).unwrap());

    let mss = MediaSourceStream::new(source, Default::default());

    // Use the default options for format readers other than for gapless playback.
    let format_opts = FormatOptions {
        enable_gapless: true,
        ..Default::default()
    };

    // Use the default options for metadata readers.
    let metadata_opts: MetadataOptions = Default::default();

    // Get the value of the track option, if provided.
    // let track = match args.value_of("track") {
    //     Some(track_str) => track_str.parse::<usize>().ok(),
    //     _ => None,
    // };

    let track = None;

    if let Ok(probed) =
        symphonia::default::get_probe().format(&hint, mss, &format_opts, &metadata_opts)
    {
        // Playback mode.
        // println!("{:?}", &mut probed);

        // If present, parse the seek argument.
        let seek_time = Some(0.0);

        // Set the decoder options.
        let decode_opts = DecoderOptions { verify: false };

        // Play it!
        _ = play(
            probed.format,
            track,
            seek_time,
            &decode_opts,
            receiver,
            sync_sender,
        );
    }
}

#[derive(Copy, Clone)]
struct PlayTrackOptions {
    track_id: u32,
    seek_ts: u64,
}

fn play(
    mut reader: Box<dyn FormatReader>,
    track_num: Option<usize>,
    seek_time: Option<f64>,
    decode_opts: &DecoderOptions,
    receiver: std::sync::mpsc::Receiver<InternalPlayerCommands>,
    sync_sender: std::sync::mpsc::Sender<PlayerUpdate>,
) -> Result<i32> {
    // If the user provided a track number, select that track if it exists, otherwise, select the
    // first track with a known codec.
    let track = track_num
        .and_then(|t| reader.tracks().get(t))
        .or_else(|| first_supported_track(reader.tracks()));

    let mut track_id = match track {
        Some(track) => track.id,
        _ => return Ok(0),
    };

    // If there is a seek time, seek the reader to the time specified and get the timestamp of the
    // seeked position. All packets with a timestamp < the seeked position will not be played.
    //
    // Note: This is a half-baked approach to seeking! After seeking the reader, packets should be
    // decoded and *samples* discarded up-to the exact *sample* indicated by required_ts. The
    // current approach will discard excess samples if seeking to a sample within a packet.
    let seek_ts = if let Some(time) = seek_time {
        let seek_to = SeekTo::Time {
            time: Time::from(time),
            track_id: Some(track_id),
        };

        // Attempt the seek. If the seek fails, ignore the error and return a seek timestamp of 0 so
        // that no samples are trimmed.
        match reader.seek(SeekMode::Accurate, seek_to) {
            Ok(seeked_to) => seeked_to.required_ts,
            Err(Error::ResetRequired) => {
                print_tracks(reader.tracks());
                track_id = first_supported_track(reader.tracks()).unwrap().id;
                0
            }
            Err(err) => {
                // Don't give-up on a seek error.
                warn!("seek error: {}", err);
                0
            }
        }
    } else {
        // If not seeking, the seek timestamp is 0.
        0
    };

    // The audio output device.
    let mut audio_output = None;

    let mut track_info = PlayTrackOptions { track_id, seek_ts };

    let mut pause = false;

    let result = loop {
        match play_track(
            &mut reader,
            &mut audio_output,
            track_info,
            decode_opts,
            &receiver,
            &mut pause,
            &sync_sender,
        ) {
            Err(Error::ResetRequired) => {
                // The demuxer indicated that a reset is required. This is sometimes seen with
                // streaming OGG (e.g., Icecast) wherein the entire contents of the container change
                // (new tracks, codecs, metadata, etc.). Therefore, we must select a new track and
                // recreate the decoder.
                print_tracks(reader.tracks());

                // Select the first supported track since the user's selected track number might no
                // longer be valid or make sense.
                let track_id = first_supported_track(reader.tracks()).unwrap().id;
                track_info = PlayTrackOptions {
                    track_id,
                    seek_ts: 0,
                };
            }
            res => break res,
        }
    };

    // Flush the audio output to finish playing back any leftover samples.
    if let Some(audio_output) = audio_output.as_mut() {
        audio_output.flush()
    }

    result
}

fn play_track(
    reader: &mut Box<dyn FormatReader>,
    audio_output: &mut Option<Box<dyn output::AudioOutput>>,
    play_opts: PlayTrackOptions,
    decode_opts: &DecoderOptions,
    receiver: &std::sync::mpsc::Receiver<InternalPlayerCommands>,
    pause: &mut bool,
    sync_sender: &std::sync::mpsc::Sender<PlayerUpdate>,
) -> Result<i32> {
    // Get the selected track using the track ID.
    let track = match reader
        .tracks()
        .iter()
        .find(|track| track.id == play_opts.track_id)
    {
        Some(track) => track,
        _ => return Ok(0),
    };

    // Create a decoder for the track.
    let mut decoder = symphonia::default::get_codecs().make(&track.codec_params, decode_opts)?;

    // Get the selected track's timebase and duration.
    let tb = track.codec_params.time_base;
    let dur = track
        .codec_params
        .n_frames
        .map(|frames| track.codec_params.start_ts + frames);

    // Decode and play the packets belonging to the selected track.
    let result = loop {
        // Jibao Loop here ....

        if let Ok(cmd) = receiver.try_recv() {
            match cmd {
                InternalPlayerCommands::Stop => break Err(Error::Unsupported("stopped")),
                InternalPlayerCommands::Pause => *pause = true,
                InternalPlayerCommands::Resume => *pause = false,
            }
        };

        if *pause {
            continue;
        }

        // Get the next packet from the format reader.
        let packet = match reader.next_packet() {
            Ok(packet) => packet,
            Err(err) => break Err(err),
        };

        // If the packet does not belong to the selected track, skip it.
        if packet.track_id() != play_opts.track_id {
            continue;
        }

        //Print out new metadata.
        while !reader.metadata().is_latest() {
            reader.metadata().pop();

            if let Some(rev) = reader.metadata().current() {
                print_update(rev);
            }
        }

        // Decode the packet into audio samples.
        match decoder.decode(&packet) {
            Ok(decoded) => {
                // If the audio output is not open, try to open it.
                if audio_output.is_none() {
                    // Get the audio buffer specification. This is a description of the decoded
                    // audio buffer's sample format and sample rate.
                    let spec = *decoded.spec();

                    // Get the capacity of the decoded buffer. Note that this is capacity, not
                    // length! The capacity of the decoded buffer is constant for the life of the
                    // decoder, but the length is not.
                    let duration = decoded.capacity() as u64;

                    // Try to open the audio output.
                    audio_output.replace(output::try_open(spec, duration).unwrap());
                } else {
                    // TODO: Check the audio spec. and duration hasn't changed.
                }

                // Write the decoded audio samples to the audio output if the presentation timestamp
                // for the packet is >= the seeked position (0 if not seeking).
                if packet.ts() >= play_opts.seek_ts {
                    print_progress(packet.ts(), dur, tb, sync_sender);

                    if let Some(audio_output) = audio_output {
                        audio_output.write(decoded).unwrap()
                    }
                }
            }
            Err(Error::DecodeError(err)) => {
                // Decode errors are not fatal. Print the error message and try to decode the next
                // packet as usual.
                warn!("decode error: {}", err);
            }
            Err(err) => break Err(err),
        }
    };

    // Return if a fatal error occured.
    ignore_end_of_stream_error(result)?;

    // Finalize the decoder and return the verification result if it's been enabled.
    do_verification(decoder.finalize())
}

fn first_supported_track(tracks: &[Track]) -> Option<&Track> {
    tracks
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
}

fn ignore_end_of_stream_error(result: Result<()>) -> Result<()> {
    match result {
        Err(Error::IoError(err))
            if err.kind() == std::io::ErrorKind::UnexpectedEof
                && err.to_string() == "end of stream" =>
        {
            // Do not treat "end of stream" as a fatal error. It's the currently only way a
            // format reader can indicate the media is complete.
            Ok(())
        }
        _ => result,
    }
}

fn do_verification(finalization: FinalizeResult) -> Result<i32> {
    match finalization.verify_ok {
        Some(is_ok) => {
            // Got a verification result.
            println!("verification: {}", if is_ok { "passed" } else { "failed" });

            Ok(i32::from(!is_ok))
        }
        // Verification not enabled by user, or unsupported by the codec.
        _ => Ok(0),
    }
}

fn print_update(rev: &MetadataRevision) {
    print_tags(rev.tags());
    print_visuals(rev.visuals());
    println!(":");
    println!();
}

fn print_tracks(tracks: &[Track]) {
    if !tracks.is_empty() {
        println!("|");
        println!("| // Tracks //");

        for (idx, track) in tracks.iter().enumerate() {
            let params = &track.codec_params;

            print!("|     [{:0>2}] Codec:           ", idx + 1);

            if let Some(codec) = symphonia::default::get_codecs().get_codec(params.codec) {
                println!("{} ({})", codec.long_name, codec.short_name);
            } else {
                println!("Unknown (#{})", params.codec);
            }

            if let Some(sample_rate) = params.sample_rate {
                println!("|          Sample Rate:     {}", sample_rate);
            }
            if params.start_ts > 0 {
                if let Some(tb) = params.time_base {
                    println!(
                        "|          Start Time:      {} ({})",
                        fmt_time(params.start_ts, tb),
                        params.start_ts
                    );
                } else {
                    println!("|          Start Time:      {}", params.start_ts);
                }
            }
            if let Some(n_frames) = params.n_frames {
                if let Some(tb) = params.time_base {
                    println!(
                        "|          Duration:        {} ({})",
                        fmt_time(n_frames, tb),
                        n_frames
                    );
                } else {
                    println!("|          Frames:          {}", n_frames);
                }
            }
            if let Some(tb) = params.time_base {
                println!("|          Time Base:       {}", tb);
            }
            if let Some(padding) = params.delay {
                println!("|          Encoder Delay:   {}", padding);
            }
            if let Some(padding) = params.padding {
                println!("|          Encoder Padding: {}", padding);
            }
            if let Some(sample_format) = params.sample_format {
                println!("|          Sample Format:   {:?}", sample_format);
            }
            if let Some(bits_per_sample) = params.bits_per_sample {
                println!("|          Bits per Sample: {}", bits_per_sample);
            }
            if let Some(channels) = params.channels {
                println!("|          Channel(s):      {}", channels.count());
                println!("|          Channel Map:     {}", channels);
            }
            if let Some(channel_layout) = params.channel_layout {
                println!("|          Channel Layout:  {:?}", channel_layout);
            }
            if let Some(language) = &track.language {
                println!("|          Language:        {}", language);
            }
        }
    }
}

fn print_tags(tags: &[Tag]) {
    if !tags.is_empty() {
        println!("|");
        println!("| // Tags //");

        let mut idx = 1;

        // Print tags with a standard tag key first, these are the most common tags.
        for tag in tags.iter().filter(|tag| tag.is_known()) {
            if let Some(std_key) = tag.std_key {
                println!(
                    "{}",
                    print_tag_item(idx, &format!("{:?}", std_key), &tag.value, 4)
                );
            }
            idx += 1;
        }

        // Print the remaining tags with keys truncated to 26 characters.
        for tag in tags.iter().filter(|tag| !tag.is_known()) {
            println!("{}", print_tag_item(idx, &tag.key, &tag.value, 4));
            idx += 1;
        }
    }
}

fn print_visuals(visuals: &[Visual]) {
    if !visuals.is_empty() {
        println!("|");
        println!("| // Visuals //");

        for (idx, visual) in visuals.iter().enumerate() {
            if let Some(usage) = visual.usage {
                println!("|     [{:0>2}] Usage:      {:?}", idx + 1, usage);
                println!("|          Media Type: {}", visual.media_type);
            } else {
                println!("|     [{:0>2}] Media Type: {}", idx + 1, visual.media_type);
            }
            if let Some(dimensions) = visual.dimensions {
                println!(
                    "|          Dimensions: {} px x {} px",
                    dimensions.width, dimensions.height
                );
            }
            if let Some(bpp) = visual.bits_per_pixel {
                println!("|          Bits/Pixel: {}", bpp);
            }
            if let Some(ColorMode::Indexed(colors)) = visual.color_mode {
                println!("|          Palette:    {} colors", colors);
            }
            println!("|          Size:       {} bytes", visual.data.len());

            // Print out tags similar to how regular tags are printed.
            if !visual.tags.is_empty() {
                println!("|          Tags:");
            }

            for (tidx, tag) in visual.tags.iter().enumerate() {
                if let Some(std_key) = tag.std_key {
                    println!(
                        "{}",
                        print_tag_item(tidx + 1, &format!("{:?}", std_key), &tag.value, 21)
                    );
                } else {
                    println!("{}", print_tag_item(tidx + 1, &tag.key, &tag.value, 21));
                }
            }
        }
    }
}

fn print_tag_item(idx: usize, key: &str, value: &Value, indent: usize) -> String {
    let key_str = match key.len() {
        0..=28 => format!("| {:w$}[{:0>2}] {:<28} : ", "", idx, key, w = indent),
        _ => format!(
            "| {:w$}[{:0>2}] {:.<28} : ",
            "",
            idx,
            key.split_at(26).0,
            w = indent
        ),
    };

    let line_prefix = format!("\n| {:w$} : ", "", w = indent + 4 + 28 + 1);
    let line_wrap_prefix = format!("\n| {:w$}   ", "", w = indent + 4 + 28 + 1);

    let mut out = String::new();

    out.push_str(&key_str);

    for (wrapped, line) in value.to_string().lines().enumerate() {
        if wrapped > 0 {
            out.push_str(&line_prefix);
        }

        let mut chars = line.chars();
        let split = (0..)
            .map(|_| chars.by_ref().take(72).collect::<String>())
            .take_while(|s| !s.is_empty())
            .collect::<Vec<_>>();

        out.push_str(&split.join(&line_wrap_prefix));
    }

    out
}

fn fmt_time(ts: u64, tb: TimeBase) -> String {
    let time = tb.calc_time(ts);

    let hours = time.seconds / (60 * 60);
    let mins = (time.seconds % (60 * 60)) / 60;
    let secs = f64::from((time.seconds % 60) as u32) + time.frac;

    format!("{}:{:0>2}:{:0>6.3}", hours, mins, secs)
}

fn print_progress(
    ts: u64,
    dur: Option<u64>,
    tb: Option<TimeBase>,
    sync_sender: &std::sync::mpsc::Sender<PlayerUpdate>,
) {
    if let Some(tb) = tb {
        let t = tb.calc_time(ts);

        let p_hours = t.seconds / (60 * 60);
        let p_mins = (t.seconds % (60 * 60)) / 60;
        let p_secs = f64::from((t.seconds % 60) as u32) + t.frac;

        if let Some(dur) = dur {
            let d = tb.calc_time(dur);

            let t_hours = d.seconds / (60 * 60);
            let t_mins = (d.seconds % (60 * 60)) / 60;
            let t_secs = f64::from((d.seconds % 60) as u32) + d.frac;

            _ = sync_sender.send(PlayerUpdate::Progress {
                position: (p_hours, p_mins, p_secs),
                total: (t_hours, t_mins, t_secs),
            });
        } else {
            _ = sync_sender.send(PlayerUpdate::Progress {
                position: (p_hours, p_mins, p_secs),
                total: Default::default(),
            });
        }
    }
}
