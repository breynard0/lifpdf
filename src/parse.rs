use crate::{SlintCompetitorRow, SlintEventRow, SlintRaceEvent, SlintSkaterTime};
use slint::VecModel;

#[derive(Clone, Debug, Default)]
pub struct EventRow {
    pub event_code: String,
    pub event_name: String,
    pub start_time: String,
}

#[derive(Clone, Debug, Default)]
pub struct SkaterTime {
    pub minutes: u32,
    pub seconds: u32,
    pub subsecond: f32,
}

#[derive(Clone, Debug, Default)]
pub struct CompetitorRow {
    pub place: Option<u8>,
    pub skater_id: Option<u32>,
    pub lane: Option<u8>,
    pub last_name: String,
    pub first_name: String,
    pub club: String,
    pub time_raw: String,
    pub time: Option<SkaterTime>,
    pub splits_raw: String,
    pub splits: Vec<SkaterTime>,
    pub start_time: String,
}

#[derive(Clone, Debug, Default)]
pub struct RaceEvent {
    pub event: EventRow,
    pub competitors: Vec<CompetitorRow>,
}

impl RaceEvent {
    pub fn parse_lif(raw: String, file_name: String) -> Result<Self, String> {
        let lines = raw.lines().collect::<Vec<_>>();

        // Get event info from first line
        let first_line_split = match lines.first() {
            None => return Err(format!("File {} has no content", file_name)),
            Some(f) => f.split(",").collect::<Vec<_>>(),
        };

        let event_row = EventRow {
            event_code: first_line_split.first().unwrap().to_string(),
            event_name: first_line_split.get(3).unwrap().to_string(),
            start_time: first_line_split.last().unwrap().to_string(),
        };

        // Assume each subsequent line is competitor data and parse accordingly
        let mut competitor_entries = vec![];
        for i in 1..lines.len() {
            // Have to do a fancy split here since the lines include commas
            let mut cur_line_split = vec![];
            let mut cur_text = String::new();
            let mut inside_quote = false;
            for c in lines.get(i).unwrap().chars() {
                if c == '\"' {
                    inside_quote = !inside_quote;
                }

                if c == ',' && !inside_quote {
                    cur_line_split.push(cur_text.clone());
                    cur_text = String::new();
                } else {
                    cur_text.push(c);
                }
            }

            // Sometimes a to_string or something will get wrapped up in here
            // This checks for that
            if cur_line_split.len() < 4 {
                continue;
            }

            let time_raw = cur_line_split.get(6).ok_or("Missing time")?.to_string();

            let splits_raw = cur_line_split.get(10).ok_or("Missing splits")?.to_string();

            let mut splits = vec![];

            let mut active = false;
            let mut cur_time = String::new();
            for c in splits_raw.chars() {
                if c == '(' {
                    active = true;
                } else if c == ')' {
                    splits.push(parse_time(cur_time)?);
                    cur_time = String::new();
                    active = false;
                } else {
                    if active {
                        cur_time.push(c);
                    }
                }
            }

            competitor_entries.push(CompetitorRow {
                place: Some(
                    cur_line_split
                        .get(0)
                        .ok_or("Missing place")?
                        .parse::<u8>()
                        .map_err(|_| "Couldn't parse place")?,
                ),
                skater_id: Some(
                    cur_line_split
                        .get(1)
                        .ok_or("Missing skater ID")?
                        .parse::<u32>()
                        .map_err(|_| "Couldn't parse skater_id")?,
                ),
                lane: Some(
                    cur_line_split
                        .get(2)
                        .ok_or("Missing lane")?
                        .parse::<u8>()
                        .map_err(|_| "Couldn't parse lane")?,
                ),
                last_name: cur_line_split
                    .get(3)
                    .ok_or("Missing last name")?
                    .to_string(),
                first_name: cur_line_split
                    .get(4)
                    .ok_or("Missing first name")?
                    .to_string(),
                club: cur_line_split.get(5).ok_or("Missing club")?.to_string(),
                time_raw: time_raw.clone(),
                time: Some(parse_time(time_raw)?),
                splits_raw,
                splits,
                start_time: cur_line_split
                    .get(11)
                    .ok_or("Missing start time")?
                    .to_string(),
            })
        }

        Ok(Self {
            event: event_row,
            competitors: competitor_entries,
        })
    }
}

impl Into<SlintRaceEvent> for RaceEvent {
    fn into(self) -> SlintRaceEvent {
        let mut slint_competitors = vec![];

        for competitor in self.competitors {
            let time = competitor.time.unwrap_or(SkaterTime {
                minutes: 0,
                seconds: 0,
                subsecond: -1.0,
            });
            slint_competitors.push(SlintCompetitorRow {
                club: competitor.club.into(),
                first_name: competitor.first_name.into(),
                lane: competitor.lane.unwrap_or(0) as i32,
                last_name: competitor.last_name.into(),
                place: competitor.place.unwrap_or(0) as i32,
                skater_id: competitor.skater_id.unwrap_or(0) as i32,
                splits: slint::ModelRc::new(VecModel::from(
                    competitor
                        .splits
                        .iter()
                        .map(|t| SlintSkaterTime {
                            minutes: t.minutes as i32,
                            seconds: t.seconds as i32,
                            subsecont: t.subsecond,
                        })
                        .collect::<Vec<_>>(),
                )),
                start_time: competitor.start_time.into(),
                time: SlintSkaterTime {
                    minutes: time.minutes as i32,
                    seconds: time.seconds as i32,
                    subsecont: time.subsecond,
                },
            })
        }

        SlintRaceEvent {
            competitors: slint::ModelRc::new(VecModel::from(slint_competitors)),
            event: SlintEventRow {
                event_code: self.event.event_code.into(),
                event_name: self.event.event_name.into(),
                start_time: self.event.start_time.into(),
            },
        }
    }
}

fn parse_time(time: String) -> Result<SkaterTime, String> {
    let mut out = SkaterTime::default();
    let mut cur_digit = String::new();

    // There should only be one colon, if there's more, that likely means a DNF
    let mut colon_count = 0;

    // All times should include a decimal point, probably DNF if not
    let mut decimal_found = false;

    for c in time.chars() {
        match c {
            ':' => {
                out.minutes = cur_digit.parse::<u32>().map_err(|_| "Error parsing time")?;
                cur_digit = String::new();
                colon_count += 1;
            }
            '.' => {
                out.seconds = cur_digit.parse::<u32>().map_err(|_| "Error parsing time")?;
                cur_digit = String::new();
                decimal_found = true;
            }
            _ => {
                cur_digit.push(c);
            }
        }
    }

    // cur_digit should now include decimal portion, so long as not DNF
    if colon_count > 1 || !decimal_found {
        out.minutes = 0;
        out.seconds = 0;
        out.subsecond = -1.0;
    } else {
        out.subsecond = cur_digit
            .parse::<u128>()
            .map_err(|_| "Error parsing time")? as f32
            / 10.0_f32.powi(cur_digit.len() as i32);
    }

    Ok(out)
}
