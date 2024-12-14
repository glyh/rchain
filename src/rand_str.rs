use rand::Rng;
use std::sync::LazyLock;

static WORDS: LazyLock<Vec<&str>> = LazyLock::new(|| {
    let dictionary = "apple bread chair table spoon knife plate cup glass phone bottle book clock plant couch pillow mirror radio wallet paper pencil ruler eraser shoes shirt pants dress towel soap comb camera guitar watch laptop candle basket broom bucket ladder helmet car truck train bus house store street bridge window faucet toilet shower hammer wrench saw drill umbrella backpack suitcase jacket trash can bed curtain blanket lamp carpet sponge glove sink brush frame ball coin toy fork oven stove microwave fridge newspaper plug outlet key sofa bench photo purse rug mat shelf tile bowl jar lock sign fan switch mug desk flag keyboard monitor charger stapler";
    dictionary.split(' ').collect()
});

fn rand_word() -> &'static str {
    &WORDS[rand::thread_rng().gen_range(0..WORDS.len())]
}
pub fn rand_string(cnt: usize) -> String {
    let mut output = "".to_string();
    for idx in 0..cnt {
        output += rand_word();
        if idx != cnt - 1 {
            output += " ";
        }
    }
    output
}
