use cnmo_parse::lparse::LParse;
use cnmo_parse::cnma::Cnma;

use anyhow::Result;
use cnmo_parse::lparse::level_data::LevelData;

fn help() {
    println!("D: Decompile <cnmb, cnms>");
    println!("PA: Print cnmA <cnma, also_stdout: bool, output>");
    println!("C: Compile <cnmb, cnms>");
    println!("S: Save <output>");
    println!("TSA: Test Saving cnmA <cnma, output>");
    println!("L: Load <input>");
    println!("PL: Print Level data <also_stdout: bool, output>");
    println!("H: Help (Prints this)");
    println!("Q: Quit");
}

fn main() -> Result<()> {
    let mut level_data = None;

    println!("Testing facility...");
    help();

    loop {
        let mut str = "".to_string();
        std::io::stdin().read_line(&mut str)?;

        match str.split_whitespace().nth(0).unwrap().to_uppercase().as_str() {
            "D" => {
                let cnmb = LParse::from_file(str.split_whitespace().nth(1).unwrap())?;
                let cnms = LParse::from_file(str.split_whitespace().nth(2).unwrap())?;
                level_data = Some(LevelData::from_lparse(&cnmb, &cnms, false)?);
            },
            "PA" => {
                let cnma = Cnma::from_file(str.split_whitespace().nth(1).unwrap())?;
                if str.split_whitespace().nth(2).unwrap().parse::<bool>()? {
                    println!("{:?}", &cnma);
                }
                std::fs::write(str.split_whitespace().nth(3).unwrap(), format!("{:?}", &cnma))?;
            },
            "S" => {
                serde_json::to_writer_pretty(std::fs::File::create(str.split_whitespace().nth(1).unwrap()).unwrap(), level_data.as_ref().unwrap())?;
            },
            "TSA" => {
                let cnma = Cnma::from_file(str.split_whitespace().nth(1).unwrap())?;
                cnma.save(str.split_whitespace().nth(2).unwrap())?;
            },
            "L" => {
                level_data = Some(serde_json::from_reader(std::fs::File::open(str.split_whitespace().nth(1).unwrap()).unwrap())?);
            },
            "PL" => {
                if str.split_whitespace().nth(1).unwrap().parse::<bool>()? {
                    println!("{:?}", level_data.as_ref().unwrap());
                }
                std::fs::write(str.split_whitespace().nth(2).unwrap(), format!("{:?}", level_data.as_ref().unwrap()))?;
            },
            "C" => {
                let version_id = level_data.as_ref().unwrap().version.get_version();
                let (mut cnmb, mut cnms) = (LParse::new(version_id).unwrap(), LParse::new(version_id).unwrap());
                level_data.as_ref().unwrap().save(&mut cnmb, &mut cnms);
                cnmb.save_to_file(str.split_whitespace().nth(1).unwrap()).expect("Can't save cnmb file!");
                cnms.save_to_file(str.split_whitespace().nth(2).unwrap()).expect("Can't save cnms file!");
            },
            "H" => {
                help();
            }
            "Q" => {
                break;
            },
            _ => {
                println!("Invalid command!");
            }
        }
    }
    Ok(())
}
