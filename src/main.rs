use std::fs::File;
use std::fs::read_to_string;
use std::io::Write;
use std::collections::HashMap;
use std::collections::HashSet;
use clap::{Parser, Subcommand};
use std::io;
use json::JsonValue;

fn get_default_season() -> u32 {
    2021
}

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,

    /// the season
    #[clap(short, long, default_value_t = get_default_season())]
    season: u32,
}

#[derive(Subcommand)]
enum Commands {
    /// shows statistics
    Show { 
        #[clap(short, long, default_value="goals")]
        what: String
    },
    /// Updates season
    Update {
    },
    /// Submit a tip
    Submit {
        /// Name of the user who submits the tip 
        #[clap(short, long)]
        whom: Option<String>,
        /// Game day, starting at 1 
        #[clap(short, long)]
        day: Option<u32>,
        /// Name of team 1 (home)
        #[clap(short='t', long)]
        team_1: Option<String>,
        /// Goals of team 1 (home)
        #[clap(short='g', long)]
        team_1_goals: Option<u32>,
        /// Goals of team 2 (guest)
        #[clap(short='G', long)]
        team_2_goals: Option<u32>
    }
}

struct Game
{
    team_1 : String,
    team_1_goals: u32,
    team_2 : String,
    team_2_goals: u32,    
}

#[derive(Debug)]
struct Submit
{
    season: u32,
    whom: Option<String>,
    day: Option<u32>,
    team_1: Option<String>,
    team_1_goals: Option<u32>,
    team_2_goals: Option<u32>
}

struct Goal
{
    team_1_goals: u32,
    team_2_goals: u32
}

struct Tip
{
    day: u32,
    team_1: String,
    team_2: String,
    goals: HashMap<String, Goal> 
}

fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Show { what } => {
            show(cli.season, what);
        },
        Commands::Update {  } => {
            update(cli.season);
        },
        Commands::Submit { whom, day , team_1, team_1_goals, team_2_goals} => {
            let mut submit = Submit {
                season: cli.season,
                whom: whom.clone(),
                day: day.clone(),
                team_1: team_1.clone(),
                team_1_goals: team_1_goals.clone(),
                team_2_goals: team_2_goals.clone()
            };
            do_submit(&mut submit);
        }

    }
}

fn fetch(season: u32) {
    let url = format!("https://api.openligadb.de/getmatchdata/bl1/{:}", season);
    let response = reqwest::blocking::get(url).unwrap().text().unwrap();

    let mut file = File::create(format!("data/season_{:}.json", season)).unwrap();
    write!(file, "{}", response).unwrap();
}

fn show(season: u32, what: &str) {
    match what {
        "goals" => show_goals(season),
        "tips" => show_tips(season),
        _ => ()
    }
}

fn show_tips(season: u32) {
    let mut players: HashSet<String> = HashSet::new();
    let tips_json = get_tips_json(season);
    for (_, tip2) in tips_json.entries() {
        let whom = String::from(tip2["whom"].as_str().unwrap());
        players.insert(whom);
    }
    let players : Vec<String> = players.into_iter().collect();


    let contents = read_to_string(format!("data/season_{:}.json", season)).unwrap();
    let season_json = json::parse(contents.as_str()).unwrap();

    let mut tips: Vec<Tip> = Vec::new();
    for game in season_json.members() {
        let day = game["group"]["groupOrderID"].as_u32().unwrap();
        let team_1 = String::from(game["team1"]["shortName"].as_str().unwrap());
        let team_2 = String::from(game["team2"]["shortName"].as_str().unwrap());

        let mut goals: HashMap<String,Goal> = HashMap::new();
        for player in &players {
            let key = format!("{:}_{:}_{:}", day, &player, team_1);

            if tips_json.has_key(&key) {
                    let team_1_goals = tips_json[&key]["team_1_goals"].as_u32().unwrap();
                let team_2_goals = tips_json[&key]["team_2_goals"].as_u32().unwrap();

                goals.insert(String::from(player), Goal { team_1_goals, team_2_goals});
            }
        }

        tips.push(Tip{day, team_1, team_2, goals});
    }


    print!("Day | {:30} | ", "Game");
    for player in &players {
        print!("{:10} | ", player);
    }
    println!("");

    for tip in tips {
        let game_name = format!("{:} vs. {:}", tip.team_1, tip.team_2);
        print!("{:3} | {:30} | ", tip.day, game_name);
        for player in &players {
            let goals = tip.goals.get(player);
            let goals_text = match goals {
                Some(v) => format!("{:}:{:}", v.team_1_goals, v.team_2_goals ),
                _ => String::from("-")
            };
            print!("{:10} | ", goals_text);
        }
        println!("");
    }
}

fn show_goals(season: u32) {
    let contents = read_to_string(format!("data/season_{:}.json", season)).unwrap();
    let parsed = json::parse(contents.as_str()).unwrap();

    let mut games: Vec<Game> = Vec::new();

    for game in parsed.members() {
        let team_1 = game["team1"]["teamName"].as_str().unwrap();
        let team_2 = game["team2"]["teamName"].as_str().unwrap();

        let is_finished = game["matchIsFinished"].as_bool().unwrap();

        if is_finished {
            let team_1_goals = game["matchResults"][0]["pointsTeam1"].as_u32().unwrap();
            let team_2_goals = game["matchResults"][0]["pointsTeam2"].as_u32().unwrap();
            /*println!(
                "{:}:{:} {:20} - {:} ",
                team_1_goals, team_2_goals, team_1, team_2
            );*/

            games.push(Game {team_1:String::from(team_1), team_1_goals, team_2: String::from(team_2), team_2_goals})

        } else {
            // println!("?:? {:20} - {:} ", team_1, team_2);
        }
    }

    let mut goals: HashMap<String,u32> = HashMap::new();
    for game in games {
        *goals.entry(game.team_1).or_insert(0) += game.team_1_goals;
        *goals.entry(game.team_2).or_insert(0) += game.team_2_goals;
    }

    let mut sortable_goals: Vec<(String, u32)> = Vec::new();
    for (team, team_goals) in goals {
        sortable_goals.push((team, team_goals));
    }

    sortable_goals.sort_by(|(_, a), (_, b)| {
        b.cmp(a)
    });

    for (team, team_goals) in sortable_goals {
        println!("{:25}: {}", team, team_goals);
    }

}

fn update(season: u32) {
    fetch(season);
    println!("updated season={:}", season);
}

fn query_u32(result: &mut Option<u32>, what: &str)
{
    while result.is_none() {
        print!("{}: ", what);
        io::stdout().flush().unwrap();
        let mut line = String::new();
        io::stdin().read_line(&mut line).unwrap(); 
        let parse_result = line.trim().parse();
        match parse_result {
            Ok(value) => { *result = Some(value); },
            _ => { println!("invalid input"); }
        }
    }
}

fn query_str(result: &mut Option<String>, what: &str)
{
    if result.is_none() {
        print!("{}: ", what);
        io::stdout().flush().unwrap();
        let mut line = String::new();
        io::stdin().read_line(&mut line).unwrap();
        *result = Some(String::from(line.trim()));
    }
}

fn do_submit(submit: &mut Submit)
{
    query_str(&mut submit.whom, "Who are you");
    query_u32(&mut submit.day, "Game day");
    query_str(&mut submit.team_1, "Team 1 (home)");
    query_u32(&mut submit.team_1_goals, "Goals team 1 (home)");
    query_u32(&mut submit.team_2_goals, "Goals team 2 (guest)");

    save_tips(&submit);
}

fn tips_filename(season: u32) -> String
{
    format!("data/season_{:}_tips.json", season)
}

fn get_tips_json(season: u32) -> JsonValue
{
    let result = read_to_string(tips_filename(season));
    if result.is_ok() {
        let contents = result.unwrap();
        json::parse(contents.as_str()).unwrap()
    }
    else {
        let mut file = File::create(tips_filename(season)).unwrap();
        file.write_all(b"{}").unwrap();
        json::parse("{}").unwrap()
    }

}

fn save_tips(submit: &Submit)
{
    let mut tips = get_tips_json(submit.season);
    let key = format!("{}_{}_{}", submit.day.unwrap(), submit.whom.as_ref().unwrap(), submit.team_1.as_ref().unwrap());
    let mut object = JsonValue::new_object();
    object["whom"] = JsonValue::from(String::from(submit.whom.as_ref().unwrap()));
    object["day"] = JsonValue::from(submit.day.unwrap());
    object["team_1"] = JsonValue::from(String::from(submit.team_1.as_ref().unwrap()));
    object["team_1_goals"] = JsonValue::from(submit.team_1_goals.unwrap());
    object["team_2_goals"] = JsonValue::from(submit.team_2_goals.unwrap());
    tips[key] = object;

    let mut file = File::create(tips_filename(submit.season)).unwrap();
    let contents = json::stringify_pretty(tips, 4);
    file.write_all(contents.as_bytes()).unwrap();
}