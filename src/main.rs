use std::fs::read_to_string;

fn main() {
    let contents = read_to_string("data/2021_21.json").unwrap();
    let parsed = json::parse(contents.as_str()).unwrap();

    for game in parsed.members() {
        let team_1 = game["team1"]["teamName"].as_str().unwrap();
        let team_2 = game["team2"]["teamName"].as_str().unwrap();

        let is_finished = game["matchIsFinished"].as_bool().unwrap();

        if is_finished {
            let team_1_goals = game["matchResults"][0]["pointsTeam1"].as_u32().unwrap();
            let team_2_goals = game["matchResults"][0]["pointsTeam2"].as_u32().unwrap();
            println!(
                "{:}:{:} {:20} - {:} ",
                team_1_goals, team_2_goals, team_1, team_2
            );
        } else {
            println!("?:? {:20} - {:} ", team_1, team_2);
        }
    }
}
