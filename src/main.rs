use plotly::{Plot, Scatter};
use plotly::common::{Line};
use std::{fs, env, process};
use chrono::{DateTime, NaiveDate, FixedOffset};

struct DataPoint {
    bpm: f32,
    date: DateTime<FixedOffset>
}

struct DayDataPair {
    date: NaiveDate,
    data: Vec<DataPoint>
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Missing directory path");
        process::exit(1);
    };

    let files = match fs::read_dir(&args[1]) {
        Ok(f)  => f,
        Err(e) => panic!("Problem opening the file: {e:?}"),
    };

    let mut all_days_data: Vec<DayDataPair> = files.into_iter().map(|f| {
        let file = match f {
            Ok(f)  => f,
            Err(e) => panic!("Problem opening the file: {e:?}"),
        };
        let contents: String = fs::read_to_string(file.path())
            .expect("Should have read a file:");

        let lines: std::str::Split<'_, char> = contents.split('\n');

        let day_data: Result<Vec<DataPoint>, _> = lines
            .filter(|d| d.starts_with("20"))
            .map(|l: &str| parse_data_point(l))
            .collect();

        match day_data {
            Ok(data)    => DayDataPair{date: data.first().unwrap().date.date_naive(), data: data},
            Err(e)      => panic!("Problem parsing the file: {e:?}"),
        }
    }).collect();

    all_days_data.sort_by(|a,b| a.date.cmp(&b.date));

    let (dates, data) =  all_days_data
        .iter()
        .map(|f| (f.date.format("%Y-%m-%d").to_string(), f.data.iter().map(|x: &DataPoint| x.bpm).sum::<f32>() / f.data.len() as f32))
        .unzip();


    let trace = Scatter::new(dates, data).line(Line::new().shape(plotly::common::LineShape::Spline).smoothing(0.5));

    let mut plot = Plot::new();
    plot.add_trace(trace);

    match std::fs::create_dir_all("./output") {
        Ok(r)  => r,
        Err(e) => panic!("Problem creating output dir: {e:?}"),
    };

    let html = plot.to_inline_html(Some("d"));
    let path = format!("./output/inline_test.html");

    match std::fs::write(path, html) {
        Ok(r)  => r,
        Err(e) => panic!("Problem writing output file: {e:?}"),
    };

    // Write standalone HTML
    let path = format!("./output/test.html");
    plot.write_html(&path);
}

type DataPointParseResult= Result<DataPoint, Box<dyn std::error::Error>>;

fn parse_data_point(line: &str) -> DataPointParseResult {
    let data: Vec<&str> = line.split(',').collect();

    let time_stamp: DateTime<FixedOffset> = DateTime::parse_from_rfc3339(data[0])?;

    let bpm: f32 = data[1].parse::<f32>()?;

    return Ok(DataPoint { bpm: bpm, date: time_stamp });
}
