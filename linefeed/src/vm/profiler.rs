use std::time::{Duration, Instant};

use rustc_hash::FxHashMap;

use crate::grammar::ast::Span;

use super::bytecode::Bytecode;

type BytecodeDiscriminant = std::mem::Discriminant<Bytecode>;

/// Runtime profiler for the Linefeed VM.
///
/// Collects timing and frequency data for instructions and source spans.
pub struct Profiler {
    /// Count of each instruction type executed
    instruction_counts: FxHashMap<BytecodeDiscriminant, u64>,
    /// Total time spent in each instruction type
    instruction_times: FxHashMap<BytecodeDiscriminant, Duration>,
    /// Representative bytecode for each discriminant (for naming)
    instruction_examples: FxHashMap<BytecodeDiscriminant, Bytecode>,

    /// Total time spent in each source span
    span_times: FxHashMap<Span, Duration>,
    /// Count of instructions executed for each source span
    span_counts: FxHashMap<Span, u64>,

    /// Function call counts by PC location
    function_counts: FxHashMap<usize, u64>,
    /// Total time in functions (exclusive, not including nested calls)
    function_times: FxHashMap<usize, Duration>,
    /// Call stack for tracking function entry/exit: (function_pc, entry_time, accumulated_child_time)
    call_stack: Vec<(usize, Instant, Duration)>,

    /// Total time spent executing
    total_time: Duration,
    /// When profiling started
    start_time: Option<Instant>,
}

impl Default for Profiler {
    fn default() -> Self {
        Self::new()
    }
}

impl Profiler {
    pub fn new() -> Self {
        Self {
            instruction_counts: FxHashMap::default(),
            instruction_times: FxHashMap::default(),
            instruction_examples: FxHashMap::default(),
            span_times: FxHashMap::default(),
            span_counts: FxHashMap::default(),
            function_counts: FxHashMap::default(),
            function_times: FxHashMap::default(),
            call_stack: Vec::new(),
            total_time: Duration::ZERO,
            start_time: None,
        }
    }

    /// Call at the start of VM execution
    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
    }

    /// Call at the end of VM execution
    pub fn stop(&mut self) {
        if let Some(start) = self.start_time.take() {
            self.total_time = start.elapsed();
        }
    }

    /// Record execution of a single instruction
    pub fn record(&mut self, bytecode: &Bytecode, span: Span, elapsed: Duration) {
        let discriminant = std::mem::discriminant(bytecode);

        // Instruction stats
        *self.instruction_counts.entry(discriminant).or_insert(0) += 1;
        *self.instruction_times.entry(discriminant).or_insert(Duration::ZERO) += elapsed;
        self.instruction_examples
            .entry(discriminant)
            .or_insert_with(|| bytecode.clone());

        // Span stats
        *self.span_counts.entry(span).or_insert(0) += 1;
        *self.span_times.entry(span).or_insert(Duration::ZERO) += elapsed;
    }

    /// Record a function call (Call instruction)
    pub fn record_call(&mut self, function_pc: usize) {
        *self.function_counts.entry(function_pc).or_insert(0) += 1;
        self.call_stack
            .push((function_pc, Instant::now(), Duration::ZERO));
    }

    /// Record a function return
    pub fn record_return(&mut self) {
        if let Some((function_pc, entry_time, child_time)) = self.call_stack.pop() {
            let total_time = entry_time.elapsed();
            // Exclusive time = total time - time spent in child calls
            let exclusive_time = total_time.saturating_sub(child_time);
            *self
                .function_times
                .entry(function_pc)
                .or_insert(Duration::ZERO) += exclusive_time;

            // Add our total time to parent's child time
            if let Some((_, _, parent_child_time)) = self.call_stack.last_mut() {
                *parent_child_time += total_time;
            }
        }
    }

    /// Generate and print the profiling report
    pub fn print_report(&self, source: &str) {
        eprintln!();
        eprintln!("================== VM Profiler Report ==================");
        eprintln!();

        self.print_instruction_stats();
        self.print_span_stats(source);
        self.print_function_stats();
        self.print_summary();

        eprintln!("=========================================================");
    }

    fn print_instruction_stats(&self) {
        let total_count: u64 = self.instruction_counts.values().sum();
        let total_time: Duration = self.instruction_times.values().sum();

        // Collect and sort by total time (descending)
        let mut stats: Vec<_> = self
            .instruction_counts
            .iter()
            .map(|(disc, &count)| {
                let time = self.instruction_times.get(disc).copied().unwrap_or_default();
                let example = self.instruction_examples.get(disc);
                let name = example.map(|b| b.name()).unwrap_or("???");
                (name, count, time)
            })
            .collect();

        stats.sort_by(|a, b| b.2.cmp(&a.2));

        eprintln!("INSTRUCTION PROFILE (by total time):");
        eprintln!(
            "  {:20} {:>12} {:>8} {:>12} {:>12}",
            "Instruction", "Count", "%", "Avg Time", "Total Time"
        );
        eprintln!("  {}", "-".repeat(68));

        for (name, count, time) in stats.iter().take(25) {
            let pct = if total_count > 0 {
                (*count as f64 / total_count as f64) * 100.0
            } else {
                0.0
            };
            let avg = if *count > 0 {
                *time / (*count as u32)
            } else {
                Duration::ZERO
            };
            eprintln!(
                "  {:20} {:>12} {:>7.1}% {:>12} {:>12}",
                name,
                format_count(*count),
                pct,
                format_duration(avg),
                format_duration(*time)
            );
        }

        if stats.len() > 25 {
            eprintln!("  ... and {} more instruction types", stats.len() - 25);
        }

        eprintln!();
        eprintln!(
            "  Total: {} instructions in {}",
            format_count(total_count),
            format_duration(total_time)
        );
        eprintln!();
    }

    fn print_span_stats(&self, source: &str) {
        if self.span_times.is_empty() {
            return;
        }

        let total_time: Duration = self.span_times.values().sum();

        // Collect and sort by total time (descending)
        let mut stats: Vec<_> = self
            .span_times
            .iter()
            .map(|(&span, &time)| {
                let count = self.span_counts.get(&span).copied().unwrap_or(0);
                (span, count, time)
            })
            .collect();

        stats.sort_by(|a, b| b.2.cmp(&a.2));

        eprintln!("SOURCE HOTSPOTS (by total time):");
        eprintln!(
            "  {:30} {:>12} {:>8} {:>12} {:>12}",
            "Location", "Count", "%", "Avg Time", "Total Time"
        );
        eprintln!("  {}", "-".repeat(78));

        for (span, count, time) in stats.iter().take(15) {
            let pct = if !total_time.is_zero() {
                time.as_secs_f64() / total_time.as_secs_f64() * 100.0
            } else {
                0.0
            };
            let avg = if *count > 0 {
                *time / (*count as u32)
            } else {
                Duration::ZERO
            };

            let location = format_span(*span, source);
            eprintln!(
                "  {:30} {:>12} {:>7.1}% {:>12} {:>12}",
                location,
                format_count(*count),
                pct,
                format_duration(avg),
                format_duration(*time)
            );
        }

        if stats.len() > 15 {
            eprintln!("  ... and {} more source locations", stats.len() - 15);
        }

        eprintln!();
    }

    fn print_function_stats(&self) {
        if self.function_counts.is_empty() {
            return;
        }

        // Collect and sort by total time (descending)
        let mut stats: Vec<_> = self
            .function_counts
            .iter()
            .map(|(&pc, &count)| {
                let time = self.function_times.get(&pc).copied().unwrap_or_default();
                (pc, count, time)
            })
            .collect();

        stats.sort_by(|a, b| b.2.cmp(&a.2));

        eprintln!("FUNCTION PROFILE (by exclusive time):");
        eprintln!(
            "  {:15} {:>12} {:>12} {:>12}",
            "Location", "Calls", "Avg Time", "Total Time"
        );
        eprintln!("  {}", "-".repeat(55));

        for (pc, count, time) in stats.iter().take(15) {
            let avg = if *count > 0 {
                *time / (*count as u32)
            } else {
                Duration::ZERO
            };
            eprintln!(
                "  @pc:{:<9} {:>12} {:>12} {:>12}",
                pc,
                format_count(*count),
                format_duration(avg),
                format_duration(*time)
            );
        }

        if stats.len() > 15 {
            eprintln!("  ... and {} more functions", stats.len() - 15);
        }

        eprintln!();
    }

    fn print_summary(&self) {
        let total_count: u64 = self.instruction_counts.values().sum();
        let total_instr_time: Duration = self.instruction_times.values().sum();

        eprintln!("SUMMARY:");
        eprintln!("  Total instructions executed: {}", format_count(total_count));
        eprintln!(
            "  Total instruction time:      {}",
            format_duration(total_instr_time)
        );
        eprintln!("  Total wall-clock time:       {}", format_duration(self.total_time));

        if total_count > 0 {
            let avg = total_instr_time / (total_count as u32);
            eprintln!("  Avg time per instruction:    {}", format_duration(avg));

            if !self.total_time.is_zero() {
                let instr_per_sec =
                    total_count as f64 / self.total_time.as_secs_f64();
                eprintln!("  Instructions per second:     {:.2}M", instr_per_sec / 1_000_000.0);
            }
        }

        eprintln!();
    }
}

/// Format a count with thousand separators
fn format_count(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

/// Format a duration in human-readable form
fn format_duration(d: Duration) -> String {
    let nanos = d.as_nanos();
    if nanos < 1_000 {
        format!("{}ns", nanos)
    } else if nanos < 1_000_000 {
        format!("{:.1}µs", nanos as f64 / 1_000.0)
    } else if nanos < 1_000_000_000 {
        format!("{:.2}ms", nanos as f64 / 1_000_000.0)
    } else {
        format!("{:.2}s", d.as_secs_f64())
    }
}

/// Format a source span as a location string
fn format_span(span: Span, source: &str) -> String {
    // Calculate line and column from byte offset
    let (line, col) = byte_offset_to_line_col(source, span.start);
    let (end_line, end_col) = byte_offset_to_line_col(source, span.end);

    if line == end_line {
        if col == end_col || span.end == span.start + 1 {
            format!("line {}:{}", line, col)
        } else {
            format!("line {}:{}-{}", line, col, end_col)
        }
    } else {
        format!("lines {}-{}", line, end_line)
    }
}

/// Convert a byte offset to (line, column), both 1-indexed
fn byte_offset_to_line_col(source: &str, offset: usize) -> (usize, usize) {
    let offset = offset.min(source.len());
    let mut line = 1;
    let mut col = 1;

    for (i, c) in source.char_indices() {
        if i >= offset {
            break;
        }
        if c == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }

    (line, col)
}
