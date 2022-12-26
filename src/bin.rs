use std::{time::Duration};

use clap::{Parser, Subcommand};
use klib::{base::{Void, Parsable}, chord::{Chordable, HasChord, Chord}, pitch::HasFrequency, octave::Octave, note::Note};
use rodio::{OutputStream, Sink, source::SineWave, Source};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Describes a chord
    Describe {
        /// Chord symbol to parse
        symbol: String,

        /// Sets the octave of the primary note
        #[arg(short, long, default_value_t = 4i8)]
        octave: i8,
    },

    /// Describes and plays a chord
    Play {
        /// Chord symbol to parse
        symbol: String,

        /// Sets the octave of the primary note
        #[arg(short, long, default_value_t = 4i8)]
        octave: i8,

        /// Sets the inversion of the chord
        #[arg(short, long, default_value_t = 0u8)]
        inversion: u8,

        /// Sets the delay between notes (in seconds)
        #[arg(short, long, default_value_t = 0.2f32)]
        delay: f32,

        /// Sets the duration of play (in seconds)
        #[arg(short, long, default_value_t = 3.0f32)]
        length: f32,

        /// Fade in duration (in seconds)
        #[arg(short, long, default_value_t = 0.1f32)]
        fade_in: f32,
        
        /// Sets whether or not the chord should be played "crunchy"
        #[arg(short, long, default_value_t = false)]
        crunchy: bool,
    },

    /// Attempt to guess the chord from a set of notes (ordered by simplicity)
    Guess {
        /// A set of notes from which the guesser will attempt to build a chord
        notes: Vec<String>,
    },
}

fn main() -> Void {
    let args = Args::parse();

    start(args)?;

    Ok(())
}

fn start(args: Args) -> Void {
    match args.command {
        Some(Command::Describe { symbol, octave }) => {
            let chord = Chord::parse(&symbol)?.with_octave(Octave::Zero + octave);

            describe(&chord);
        },
        Some(Command::Play { symbol, octave, inversion, delay, length, fade_in, crunchy }) => {
            let chord = Chord::parse(&symbol)?.with_octave(Octave::Zero + octave).with_inversion(inversion).with_crunchy(crunchy);

            play(&chord, delay, length, fade_in)?;
        },
        Some(Command::Guess { notes }) => {
            // Parse the notes.
            let notes = notes.into_iter().map(|n| Note::parse(&n)).collect::<Result<Vec<_>, _>>()?;

            // Get the chord from the notes.
            let candidates = Chord::from_notes(&notes)?;

            for candidate in candidates {
                describe(&candidate);
            }
        },
        None => {
            println!("No command given.");
        }
    }
    Ok(())
}

fn describe(chord: &Chord) {
    println!("{}", chord);
}

fn play(chord: &Chord, delay: f32, length: f32, fade_in: f32) -> Void {
    describe(chord);

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();

    let mut sinks = vec![];

    let chord_tones = chord.chord();

    if length <= chord_tones.len() as f32 * delay {
        return Err(anyhow::Error::msg("The delay is too long for the length of play (i.e., the number of chord tones times the delay is longer than the length)."));
    }

    for (k, n) in chord_tones.into_iter().enumerate() {
        let sink = Sink::try_new(&stream_handle).unwrap();

        let d = k as f32 * delay;

        let source = SineWave::new(n.frequency())
            .take_duration(Duration::from_secs_f32(length - d))
            .buffered()
            .delay(Duration::from_secs_f32(d))
            .fade_in(Duration::from_secs_f32(fade_in))
            .amplify(0.20);

        sink.append(source);

        sinks.push(sink);
    }

    std::thread::sleep(Duration::from_secs_f32(length));

    Ok(())
}

// Tests.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_describe() {
        start(Args {
            command: Some(Command::Describe {
                symbol: "Cmaj7".to_string(),
                octave: 4,
            }),
        }).unwrap();
    }

    #[test]
    fn test_guess() {
        start(Args {
            command: Some(Command::Guess {
                notes: vec!["C".to_owned(), "E".to_owned(), "G".to_owned()],
            }),
        }).unwrap();
    }
}