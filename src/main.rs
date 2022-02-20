use std::fs::File;
use std::io::Write;
use std::collections::HashMap;

struct Game
{
    team_1 : String,
    team_1_goals: u32,
    team_2 : String,
    team_2_goals: u32,    
}

fn fetch(season: u32) -> String {
    let url = format!("https://api.openligadb.de/getmatchdata/bl1/{:}", season);
    let response = reqwest::blocking::get(url).unwrap().text().unwrap();

    let mut file = File::create(format!("data/season_{:}.json", season)).unwrap();
    write!(file, "{}", response).unwrap();
    response
}

fn main() {
    let contents = fetch(2021);


    // let contents = read_to_string("data/2021_21.json").unwrap();
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
