use crate::timeline::FrameTime;
use anyhow::{anyhow, Context, Result};

/// 时间跳转解析器
pub struct GotoTimeParser;

impl GotoTimeParser {
    /// 解析用户输入的 goto 时间目标
    /// 规则（根据 PLAN.md）：
    /// 1. 相对时间：以 `+` 或 `-` 开头，例如 `+1:15.3` (向后 1分15秒300毫秒), `-30` (向前 30秒)
    /// 2. `10` 或 `10.12`：指跳转到距离当前位置最近的 ?分10秒 或 ?分10.12秒（例如当前 1:50，距离 1:10 是 40s，距离 2:10 是 20s，因此跳到 2:10）
    /// 3. `1:10` 或 `1:10.5`：指跳转到距离当前位置最近的 ?小时1分10秒
    /// 4. `1:05:20`：绝对时间定位 1小时5分20秒
    pub fn parse(input: &str, current_playhead: FrameTime) -> Result<FrameTime> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Err(anyhow!("Empty time string"));
        }

        // 1. 相对时间 (+ / -)
        if trimmed.starts_with('+') || trimmed.starts_with('-') {
            let is_positive = trimmed.starts_with('+');
            let time_part = &trimmed[1..];
            let delta_us = parse_duration_us(time_part)?;
            let current_us = current_playhead.0;
            let target_us = if is_positive {
                current_us.saturating_add(delta_us)
            } else {
                current_us.saturating_sub(delta_us).max(0)
            };
            return Ok(FrameTime(target_us));
        }

        // 2. 包含冒号的格式 (分:秒 或 时:分:秒)
        let parts: Vec<&str> = trimmed.split(':').collect();
        match parts.len() {
            1 => {
                // 单个数字 (例如 "10" 或 "10.12")
                // 跳转到距离当前位置最近的 (? 分钟 + seconds)
                let target_sec: f64 = parts[0]
                    .parse()
                    .with_context(|| format!("Invalid seconds format: {}", parts[0]))?;
                if !(0.0..60.0).contains(&target_sec) {
                    // 如果超过60秒，按纯秒数处理
                    let us = (target_sec * 1_000_000.0).round() as i64;
                    return Ok(FrameTime(us.max(0)));
                }

                let target_sec_us = (target_sec * 1_000_000.0).round() as i64;
                let current_us = current_playhead.0;
                let minute_us = 60 * 1_000_000;

                let current_min_base = (current_us / minute_us) * minute_us;

                // 候选 1: 前一分钟
                let c1 = current_min_base - minute_us + target_sec_us;
                // 候选 2: 当前分钟
                let c2 = current_min_base + target_sec_us;
                // 候选 3: 后一分钟
                let c3 = current_min_base + minute_us + target_sec_us;

                let candidates = [c1, c2, c3];
                let mut best = c2;
                let mut min_diff = i64::MAX;

                for &c in &candidates {
                    if c >= 0 {
                        let diff = (c - current_us).abs();
                        if diff < min_diff {
                            min_diff = diff;
                            best = c;
                        }
                    }
                }

                Ok(FrameTime(best))
            }
            2 => {
                // "分:秒" (例如 "1:10" 或 "1:10.5")
                // 跳转到距离当前位置最近的 (? 小时 + 分:秒)
                let mins: i64 = parts[0]
                    .parse()
                    .with_context(|| format!("Invalid minutes format: {}", parts[0]))?;
                let secs: f64 = parts[1]
                    .parse()
                    .with_context(|| format!("Invalid seconds format: {}", parts[1]))?;

                let pattern_us = mins * 60 * 1_000_000 + (secs * 1_000_000.0).round() as i64;
                let hour_us = 3600 * 1_000_000;
                let current_us = current_playhead.0;
                let current_hour_base = (current_us / hour_us) * hour_us;

                let c1 = current_hour_base - hour_us + pattern_us;
                let c2 = current_hour_base + pattern_us;
                let c3 = current_hour_base + hour_us + pattern_us;

                let candidates = [c1, c2, c3];
                let mut best = c2;
                let mut min_diff = i64::MAX;

                for &c in &candidates {
                    if c >= 0 {
                        let diff = (c - current_us).abs();
                        if diff < min_diff {
                            min_diff = diff;
                            best = c;
                        }
                    }
                }

                Ok(FrameTime(best))
            }
            3 => {
                // "时:分:秒" (例如 "1:05:20") 绝对定位
                let hours: i64 = parts[0]
                    .parse()
                    .with_context(|| format!("Invalid hours: {}", parts[0]))?;
                let mins: i64 = parts[1]
                    .parse()
                    .with_context(|| format!("Invalid minutes: {}", parts[1]))?;
                let secs: f64 = parts[2]
                    .parse()
                    .with_context(|| format!("Invalid seconds: {}", parts[2]))?;

                let total_us = hours * 3600 * 1_000_000
                    + mins * 60 * 1_000_000
                    + (secs * 1_000_000.0).round() as i64;
                Ok(FrameTime(total_us.max(0)))
            }
            _ => Err(anyhow!("Unsupported time format: {}", input)),
        }
    }
}

fn parse_duration_us(s: &str) -> Result<i64> {
    let parts: Vec<&str> = s.split(':').collect();
    match parts.len() {
        1 => {
            let secs: f64 = parts[0]
                .parse()
                .with_context(|| format!("Invalid relative seconds: {}", parts[0]))?;
            Ok((secs * 1_000_000.0).round() as i64)
        }
        2 => {
            let mins: i64 = parts[0]
                .parse()
                .with_context(|| format!("Invalid relative minutes: {}", parts[0]))?;
            let secs: f64 = parts[1]
                .parse()
                .with_context(|| format!("Invalid relative seconds: {}", parts[1]))?;
            Ok(mins * 60 * 1_000_000 + (secs * 1_000_000.0).round() as i64)
        }
        3 => {
            let hours: i64 = parts[0]
                .parse()
                .with_context(|| format!("Invalid relative hours: {}", parts[0]))?;
            let mins: i64 = parts[1]
                .parse()
                .with_context(|| format!("Invalid relative minutes: {}", parts[1]))?;
            let secs: f64 = parts[2]
                .parse()
                .with_context(|| format!("Invalid relative seconds: {}", parts[2]))?;
            Ok(hours * 3600 * 1_000_000
                + mins * 60 * 1_000_000
                + (secs * 1_000_000.0).round() as i64)
        }
        _ => Err(anyhow!("Invalid duration format: {}", s)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_goto_nearest_ten_seconds() {
        // 当前在 1分50秒 (110秒)
        let cur = FrameTime(110 * 1_000_000);
        // goto 10 -> 距离 1:10 (70s) 是 40s，距离 2:10 (130s) 是 20s
        // 应跳转到 2分10秒 (130秒)
        let res = GotoTimeParser::parse("10", cur).unwrap();
        assert_eq!(res, FrameTime(130 * 1_000_000));

        // 当前在 1分05秒 (65秒)
        let cur2 = FrameTime(65 * 1_000_000);
        // goto 10 -> 距离 1:10 (70s) 是 5s，距离 0:10 (10s) 是 55s
        // 应跳转到 1分10秒 (70秒)
        let res2 = GotoTimeParser::parse("10", cur2).unwrap();
        assert_eq!(res2, FrameTime(70 * 1_000_000));
    }

    #[test]
    fn test_goto_decimal_seconds() {
        // goto 10.120 -> 10秒120毫秒
        let cur = FrameTime(0);
        let res = GotoTimeParser::parse("10.12", cur).unwrap();
        assert_eq!(res, FrameTime(10_120_000));
    }

    #[test]
    fn test_goto_nearest_minutes_seconds() {
        // goto 1:10 -> 最近的 ?h 1m 10s
        let cur = FrameTime(3600 * 1_000_000 + 55 * 60 * 1_000_000); // 1小时55分
                                                                     // 距离 1小时1分10秒是 53分50秒，距离 2小时1分10秒是 6分10秒
        let res = GotoTimeParser::parse("1:10", cur).unwrap();
        assert_eq!(res, FrameTime(2 * 3600 * 1_000_000 + 70 * 1_000_000));
    }

    #[test]
    fn test_goto_relative_forward_and_backward() {
        let cur = FrameTime(100 * 1_000_000); // 100s
                                              // +1:15.3 -> + 75.3s = 175.3s
        let res_pos = GotoTimeParser::parse("+1:15.3", cur).unwrap();
        assert_eq!(res_pos, FrameTime(175_300_000));

        // -30 -> - 30s = 70s
        let res_neg = GotoTimeParser::parse("-30", cur).unwrap();
        assert_eq!(res_neg, FrameTime(70_000_000));
    }

    #[test]
    fn test_goto_absolute_hours() {
        let cur = FrameTime(0);
        let res = GotoTimeParser::parse("1:05:20", cur).unwrap();
        let expected = (3600 + 5 * 60 + 20) * 1_000_000;
        assert_eq!(res, FrameTime(expected));
    }
}
