use crate::parse::CompetitorRow;

pub fn gen_table_row(competitor: CompetitorRow) -> Vec<String> {
    let mut out = vec![];

    out.push(match competitor.place.unwrap() {
        255 => "DNF".to_string(),
        _ => competitor.place.unwrap().to_string(),
    });

    out.push(match competitor.skater_id.unwrap() as i32 {
        i32::MAX => "Missing".to_string(),
        _ => competitor.skater_id.unwrap().to_string(),
    });

    out.push(match competitor.lane.unwrap() {
        255 => "Missing".to_string(),
        _ => competitor.lane.unwrap().to_string(),
    });

    out.push(competitor.first_name);
    out.push(competitor.last_name);
    out.push(competitor.club);

    let mut time = competitor.time.unwrap().to_string();
    if competitor.time.unwrap().minutes == 0
        && competitor.time.unwrap().seconds == 0
        && competitor.time.unwrap().subsecond == -1.0
    {
        time = "No Time".to_string();
    }
    out.push(time);

    out
}
