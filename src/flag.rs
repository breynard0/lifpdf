use crate::parse::SkaterTime;

pub fn is_time_discrepancy(pf_time: SkaterTime, splits: &Vec<SkaterTime>) -> bool {
    let mut lowest_difference = 1000000.0;
    let mut cumulative = SkaterTime::default();
    for split in splits {
        cumulative = cumulative + *split;
        let diff = cumulative.absolute_difference_secs(pf_time);
        lowest_difference = diff.min(lowest_difference);
    }

    lowest_difference > 0.4
}
