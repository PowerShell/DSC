// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::fmt;

use crate::{
    StderrFormat,
    tracing::{Event, Level, Subscriber},
    tracing_log::NormalizeEvent,
    tracing_subscriber::{
        fmt::{
            FmtContext, FormatEvent, FormatFields,
            format::Writer,
            time::{FormatTime, SystemTime},
        },
        registry::LookupSpan,
    },
};

/// Preserves JSON events and formats text events without span names or span fields.
///
/// The containing layer controls ANSI styling and must use JSON field formatting
/// when `format` is [`StderrFormat::Json`].
pub struct DscTracingEventFormatter {
    pub with_source: bool,
    pub format: StderrFormat,
}

impl<S, N> FormatEvent<S, N> for DscTracingEventFormatter
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
    N: for<'writer> FormatFields<'writer> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        // If the format is JSON, delegate to the upstream JSON formatter.
        if self.format == StderrFormat::Json {
            return crate::tracing_subscriber::fmt::format()
                .json()
                .with_target(self.with_source)
                .with_line_number(self.with_source)
                .format_event(ctx, writer, event);
        }

        // The upstream full formatter's styling helpers are private, so duplicate them here.
        let (dimmed, reset) = if writer.has_ansi_escapes() {
            ("\x1b[2m", "\x1b[0m")
        } else {
            ("", "")
        };
        writer.write_str(dimmed)?;
        SystemTime.format_time(&mut writer)?;
        write!(writer, "{reset} ")?;

        let normalized_metadata = event.normalized_metadata();
        let metadata = normalized_metadata
            .as_ref()
            .unwrap_or_else(|| event.metadata());
        let color = if writer.has_ansi_escapes() {
            match *metadata.level() {
                Level::TRACE => "\x1b[35m",
                Level::DEBUG => "\x1b[34m",
                Level::INFO => "\x1b[32m",
                Level::WARN => "\x1b[33m",
                Level::ERROR => "\x1b[31m",
            }
        } else {
            ""
        };
        write!(writer, "{color}{:>5}{reset} ", metadata.level())?;

        if self.with_source {
            write!(
                writer,
                "{dimmed}{}{reset}{dimmed}:{reset} ",
                metadata.target()
            )?;
        }
        if self.with_source && let Some(line) = metadata.line() {
            write!(writer, "{dimmed}{line}:{reset} ")?;
        }

        ctx.format_fields(writer.by_ref(), event)?;
        writeln!(writer)
    }
}
