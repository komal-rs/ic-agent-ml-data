use candid::Principal;
use ic_agent::Agent;
use individual_user_template::Result5;

include!(concat!(env!("OUT_DIR"), "/did/mod.rs"));

use crate::individual_user_template::IndividualUserTemplate;

pub async fn init_agent() -> Agent {
    // let pk = env::var("RECLAIM_CANISTER_PEM").expect("$RECLAIM_CANISTER_PEM is not set");

    // let identity = match ic_agent::identity::BasicIdentity::from_pem(
    //     stringreader::StringReader::new(pk.as_str()),
    // ) {
    //     Ok(identity) => identity,
    //     Err(err) => {
    //         panic!("Unable to create identity, error: {:?}", err);
    //     }
    // };

    let identity = match ic_agent::identity::Secp256k1Identity::from_pem_file(
        "/Users/komalsai/Downloads/generated-id.pem",
    ) {
        Ok(identity) => identity,
        Err(err) => {
            panic!("Unable to create identity, error: {:?}", err);
        }
    };

    let agent = match Agent::builder()
        .with_url("https://a4gq6-oaaaa-aaaab-qaa4q-cai.raw.ic0.app/") // https://a4gq6-oaaaa-aaaab-qaa4q-cai.raw.ic0.app/
        .with_identity(identity)
        .build()
    {
        Ok(agent) => agent,
        Err(err) => {
            panic!("Unable to create agent, error: {:?}", err);
        }
    };

    // ‼️‼️comment below line in mainnet‼️‼️
    // agent.fetch_root_key().await.unwrap();

    agent
}

pub fn individual_user(agent: &Agent, user_canister: Principal) -> IndividualUserTemplate<'_> {
    IndividualUserTemplate(user_canister, agent)
}

#[tokio::main]
async fn main() {
    // read canisters.json file path "../canisters.json" which is a Vec<String> of canister_ids
    let canisters =
        std::fs::read_to_string("/Users/komalsai/learning/cf-exp/ic-agent-ml-data/canisters.json")
            .unwrap();
    let canisters: Vec<String> = serde_json::from_str(&canisters).unwrap();
    let mut wtr =
        csv::Writer::from_path("/Users/komalsai/learning/cf-exp/ic-agent-ml-data/ml_data.csv")
            .unwrap();

    wtr.write_record(&["canister_id", "post_id", "like_count", "view_count"])
        .unwrap();

    let agent = init_agent().await;

    let mut canister_cnt = 0;

    for canister_id in canisters.iter() {
        // canister_id format : <id>.raw.icp0.io
        let canister_id = canister_id.split('.').next().map(String::from).unwrap();
        // println!("canister_id: {:?}", canister_id);

        let canister_id = Principal::from_text(canister_id).unwrap();
        let user = individual_user(&agent, canister_id);

        // keep fetching posts in batches of 50

        let mut cursor = 0;
        loop {
            let res = match user
                .get_posts_of_this_user_profile_with_pagination_cursor(cursor, 50)
                .await
            {
                Ok(res) => res,
                Err(err) => {
                    println!(
                        "get_posts_of_this_user_profile_with_pagination_cursor Error for {}",
                        canister_id
                    );
                    break;
                }
            };

            let result = match res {
                Result5::Ok(posts) => posts,
                Result5::Err(err) => {
                    // println!("endoflist Error for {}", canister_id);
                    break;
                }
            };

            if result.len() == 0 {
                break;
            }

            for post in result.iter() {
                // println!(
                //     "canister_id: {:?}, id {:?} , likes: {:?}, views: {:?}",
                //     canister_id.to_text(),
                //     post.id,
                //     post.like_count,
                //     post.total_view_count
                // );

                wtr.write_record(&[
                    canister_id.to_text().as_str(),
                    post.id.to_string().as_str(),
                    post.like_count.to_string().as_str(),
                    post.total_view_count.to_string().as_str(),
                ])
                .unwrap_or_default();
            }

            cursor += 50;
        }

        canister_cnt += 1;
        if canister_cnt % 100 == 0 {
            println!("canister_cnt: {:?}", canister_cnt);
            wtr.flush().unwrap();
        }
    }
}
