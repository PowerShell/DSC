// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

pub(super) mod helpers {
    use std::io;
    use std::sync::{Arc, Mutex};
    #[derive(Clone, Default)]
    pub(super) struct Buffer(Arc<Mutex<Vec<u8>>>);

    impl io::Write for Buffer {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(bytes);
            Ok(bytes.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    impl Buffer {
        pub(super)fn text(&self) -> String {
            String::from_utf8(self.0.lock().unwrap().clone()).unwrap()
        }
    }

    pub(super) fn emit_levels() {
        use dsc_lib_telemetry::tracing;
        tracing::error!(target: "test_target", answer = 42, "test message");
        tracing::warn!(target: "test_target", answer = 42, "test message");
        tracing::info!(target: "test_target", answer = 42, "test message");
        tracing::debug!(target: "test_target", answer = 42, "test message");
        tracing::trace!(target: "test_target", answer = 42, "test message");
    }
}

mod upstream_compatibility {
    use test_case::test_case;
    use dsc_lib_telemetry::{DscTracingEventFormatter, StderrFormat, tracing, tracing_subscriber};
    use dsc_lib_telemetry::tracing_subscriber::prelude::*;
    
    use super::helpers::*;

    #[test_case(false, false; "without ansi and without source")]
    #[test_case(false, true; "without ansi and with source")]
    #[test_case(true, false; "with ansi and without source")]
    #[test_case(true, true; "with ansi and with source")]
    fn text_matches_upstream_without_spans(ansi: bool, with_source: bool) {
        let actual = Buffer::default();
        let expected = Buffer::default();
        let actual_writer = actual.clone();
        let expected_writer = expected.clone();
        let subscriber = tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .with_ansi(ansi)
                    .event_format(DscTracingEventFormatter {
                        with_source: with_source,
                        format: if ansi {
                            StderrFormat::Default
                        } else {
                            StderrFormat::Plaintext
                        },
                    })
                    .with_writer(move || actual_writer.clone()),
            )
            .with(
                tracing_subscriber::fmt::layer()
                    .with_ansi(ansi)
                    .with_target(with_source)
                    .with_line_number(with_source)
                    .with_writer(move || expected_writer.clone()),
            );
        tracing::subscriber::with_default(subscriber, emit_levels);
        let actual = actual.text();
        let expected = expected.text();
        assert_eq!(actual.lines().count(), 5);
        for (actual, expected) in actual.lines().zip(expected.lines()) {
            // Only the independently generated timestamps may differ.
            pretty_assertions::assert_eq!(
                actual.split_once(' ').unwrap().1,
                expected.split_once(' ').unwrap().1
            );
        }
    }

    #[test_case(StderrFormat::Default; "default format")]
    #[test_case(StderrFormat::Plaintext; "plaintext format")]
    fn text_omits_contextual_and_explicit_parent_spans(format: StderrFormat) {
        let output = Buffer::default();
        let writer = output.clone();
        let subscriber = tracing_subscriber::registry().with(
            tracing_subscriber::fmt::layer()
                .with_ansi(format == StderrFormat::Default)
                .event_format(DscTracingEventFormatter {
                    with_source: true,
                    format,
                })
                .with_writer(move || writer.clone()),
        );
        tracing::subscriber::with_default(subscriber, || {
            let root = tracing::info_span!("root_span", secret = "span_only");
            let _root = root.enter();
            let child = tracing::info_span!("child_span", nested = 7);
            {
                let _child = child.enter();
                tracing::debug!(answer = 42, "contextual event");
            }
            tracing::debug!(parent: &child, answer = 43, "explicit parent event");
            tracing::debug!(parent: None, answer = 44, "unparented event");
        });
        let output = output.text();
        assert_eq!(output.lines().count(), 3);
        for text in ["root_span", "child_span", "secret", "span_only", "nested"] {
            assert!(!output.contains(text), "{output}");
        }
        for text in [
            "contextual event",
            "explicit parent event",
            "unparented event",
            "42",
            "43",
            "44",
        ] {
            assert!(output.contains(text), "{output}");
        }
        assert_eq!(output.contains('\x1b'), format == StderrFormat::Default);
    }

    #[test_case(false; "without source")]
    #[test_case(true; "with source")]
    fn json_matches_upstream_including_span_fields(with_source: bool) {
        let actual = Buffer::default();
        let expected = Buffer::default();
        let actual_writer = actual.clone();
        let expected_writer = expected.clone();
        let subscriber = tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .json()
                    .with_ansi(false)
                    .event_format(DscTracingEventFormatter {
                        with_source: with_source,
                        format: StderrFormat::Json,
                    })
                    .with_writer(move || actual_writer.clone()),
            )
            .with(
                tracing_subscriber::fmt::layer()
                    .json()
                    .with_ansi(false)
                    .with_target(with_source)
                    .with_line_number(with_source)
                    .with_writer(move || expected_writer.clone()),
            );
        tracing::subscriber::with_default(subscriber, || {
            let root = tracing::info_span!("root_span", root_field = 1);
            let _root = root.enter();
            let child = tracing::info_span!("child_span", child_field = 2);
            {
                let _child = child.enter();
                emit_levels();
            }
            tracing::info!(parent: &child, "explicit parent event");
            tracing::info!(parent: None, "unparented event");
        });
        let actual = actual.text();
        let expected = expected.text();
        pretty_assertions::assert_eq!(actual.lines().count(), 7);
        for (actual, expected) in actual.lines().zip(expected.lines()) {
            assert!(actual.starts_with("{\"timestamp\":\""));
            pretty_assertions::assert_eq!(
                actual.split_once(',').unwrap().1,
                expected.split_once(',').unwrap().1
            );
        }
        for text in [
            "\"span\":",
            "\"spans\":",
            "\"root_field\":1",
            "\"child_field\":2",
        ] {
            assert!(actual.contains(text), "{actual}");
        }
    }
}