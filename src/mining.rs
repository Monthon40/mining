mod benmark;

extern crate chashmap;
extern crate itertools;
extern crate rayon;
extern crate reqwest;
use std::io::Read;
use std::path::PathBuf;
use error_chain::error_chain;
use itertools::Itertools;
use rayon::iter::*;
use std::fs::File;
use std::io::copy;


error_chain! {
    foreign_links {
        Io(std::io::Error);
        HttpRequest(reqwest::Error);
    }
}

pub fn ontime_rank(filename: &str) -> Vec<(String, f64)> {
    let mut fi = File::open(filename).unwrap();
    let mut infor = String::new();
    fi.read_to_string(&mut infor).ok();
    let ttt: Vec<&str> = infor.lines().skip(1).collect();
    let perform: Vec<(String, i64)> = ttt.par_iter().map(|line| helper(line.to_string())).collect();
    let information = perform.into_iter().into_group_map();
    let numValue: Vec<(&str, f64)> = information.par_iter().map(|airline| checkTime(airline.0, airline.1)).collect();
    return sorted(numValue).par_iter().map(|airline| (airline.0.to_string(), airline.1)).collect();

}
fn mix<'a>(x: Vec<(&'a str, f64)>, y: Vec<(&'a str, f64)>) -> Vec<(&'a str, f64)> {
    let mut vec = Vec::new();
    let (mut ttt, mut rrr) = (x.as_slice(), y.as_slice());
    while !(ttt.is_empty() || rrr.is_empty()) {
        if ttt[0].1 > rrr[0].1 {
            vec.push(rrr[0].clone());
            rrr = &rrr[1..];
        } else {
            vec.push(ttt[0].clone());
            ttt = &ttt[1..];
        }
    }

    if !ttt.is_empty() {
        vec.extend(ttt);
    } else {
        vec.extend(rrr);
    }
    return vec;
}

fn helper(input: String) -> (String, i64) {
    let value: Vec<&str> = input[..75].split(',').take(15).collect();
    let duration = match value[14].trim().parse::<i64>() {
        Ok(val) => val,
        Err(_err) => 0i64,
    };
    return (String::from(value[8].trim()), duration);
}
fn sorted(xs: Vec<(&str, f64)>) -> Vec<(&str, f64)> {
    return xs.par_iter().cloned().map(|val| vec![val])
        .reduce(|| vec![], |ttt, rrr| mix(ttt, rrr));
}

fn checkTime<'a>(airline: &'a str, xs: &'a Vec<i64>) -> (&'a str, f64) {
    let length = xs.len() as i64;
    let ttt: i64 = xs.to_vec().par_iter().map(|state| if state.clone() > 0 { 0 } else { 1 }).reduce(|| 0, |a, b| a + b);
    return (airline.clone(), (ttt as f64 / length as f64) * 100f64);
}

async fn download(path: PathBuf) -> Result<()> {
    let destination = "https://cs.muic.mahidol.ac.th/~ktangwon/2008.csv.zip";
    let result = reqwest::get(destination).await?;

    let mut place = {
        let filename = result.url().path_segments().and_then(|segments| segments.last()).and_then(|name| if name.is_empty() { None } else { Some(name) }).unwrap_or("tmp.bin");
        println!("Can't download file: '{}'", filename);
        let name = path.join(filename);
        println!("File is at: {:?}", name);
        File::create(name)?
    };
    let bab = result.bytes().await?;
    copy(&mut bab.as_ref(), &mut place)?;
    Ok(())
}

fn unzip(path: &mut PathBuf) -> zip::result::ZipResult<()> {
    let mut location = File::open(path.join("2008.csv.zip"))?;
    let mut buff = Vec::new();
    location.read_to_end(&mut buff)?;
    let mut zipped = zip::ZipArchive::new(location)?;
    let mut filecsv = zipped.by_index(0)?;
    println!("Unzipping: {:?}", filecsv.name());
    let mut csv_place = File::create(path.join("2008.csv"))?;
    copy(&mut filecsv, &mut csv_place)?;
    Ok(())
}

#[tokio::main]
 async fn main() -> Result<()> {
    let mut buff = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    buff.push("resources/data/");
    let path = buff.clone();

    match download(path.clone()).await {
        Ok(_) => println!("Download Success"),
        Err(err) => println!("Download failed: {:?}", err),
    };

    match unzip(&mut path.clone()) {
        Ok(_) => println!("Unzipped Successfully"),
        Err(err) => println!("Error unzipping file: {:?}", err),
    };

    buff.push("2008.csv");
    let file_csv = buff.clone().into_os_string().into_string().ok();
    let csv_string = file_csv.as_deref().unwrap_or("");

    let last = ontime_rank(&csv_string);

    for airline in last {
        println!("Airline: {:?}, Percentage: {:?} %", airline.0, airline.1);
    }

    Ok(())
}