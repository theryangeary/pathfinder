use pathfinder::game::{Board, GameEngine};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a test word list
    let words = vec![
        "cat", "dog", "test", "word", "game", "path", "tile", "board", "day", "days", "year",
        "data", "tome", "camp", "temp", "maps", "stem", "step", "pets", "set", "net", "ten", "end",
        "den", "pen", "get", "gem", "leg", "let", "met", "map", "tap", "pat", "sat", "rat", "tar",
        "art", "car", "arc", "cap", "can", "man", "pan", "tan", "eat", "tea", "ate", "eta", "ace",
        "age", "ago", "ego", "log", "cog", "god", "nod", "don", "con", "cod", "dot", "got", "hot",
        "hop", "top", "pot", "rot", "lot", "not", "oat", "oak", "ask", "ark", "air", "are", "ear",
        "era", "ore", "roe", "row", "sow", "sew", "new", "now", "own", "won", "one", "eon", "ion",
        "son", "sun", "run", "gun", "gum", "rum", "sum", "sim", "sin", "tin", "win", "wit", "bit",
        "bat", "bag", "big", "dig", "fig", "fag", "far", "bar", "bad", "dad", "sad", "mad", "had",
        "has", "his", "hit", "kit", "lit", "pit", "sit", "fit", "fat", "hat", "hag", "lag", "tag",
        "gag", "gap", "gas", "was", "saw", "paw", "raw", "ram", "jam", "ham", "dam", "damp",
        "clamp", "stamp", "tramp", "cramp",
    ];

    // Create game engine
    let engine = GameEngine::new(words);

    // Create a test board
    let mut board = Board::new();
    // c a t e
    // o m p l
    // * s e *  (wildcards at 2,0 and 2,3)
    // r n d g
    board.set_tile(0, 0, 'c', 2, false);
    board.set_tile(0, 1, 'a', 1, false);
    board.set_tile(0, 2, 't', 1, false);
    board.set_tile(0, 3, 'e', 1, false);
    board.set_tile(1, 0, 'o', 1, false);
    board.set_tile(1, 1, 'm', 2, false);
    board.set_tile(1, 2, 'p', 2, false);
    board.set_tile(1, 3, 'l', 1, false);
    board.set_tile(2, 0, '*', 0, true); // wildcard
    board.set_tile(2, 1, 's', 1, false);
    board.set_tile(2, 2, 'e', 1, false);
    board.set_tile(2, 3, '*', 0, true); // wildcard
    board.set_tile(3, 0, 'r', 1, false);
    board.set_tile(3, 1, 'n', 1, false);
    board.set_tile(3, 2, 'd', 1, false);
    board.set_tile(3, 3, 'g', 2, false);

    println!("Demo: Finding optimal 5 words for pathfinder game");
    println!("Board layout:");
    println!("c a t e");
    println!("o m p l");
    println!("* s e *");
    println!("r n d g");
    println!();

    // Find optimal 5 words
    let result = engine.find_best_n_words(&board, 5).await?;
    let (best_words, metadata) = result;

    println!(
        "Successfully found {} words with total score: {}",
        best_words.len(),
        metadata.total_score
    );
    println!("Individual word scores: {:?}", metadata.individual_scores);
    println!();

    // Demonstrate constraint handling
    println!("Verifying wildcard constraints are compatible...");
    let constraint_check =
        pathfinder::game::board::constraints::AnswerGroupConstraintSet::is_valid_set(
            best_words.clone(),
        );
    println!(
        "Constraint validation: {}",
        if constraint_check {
            "✓ Valid"
        } else {
            "✗ Invalid"
        }
    );
    println!();

    // Also demonstrate finding all valid words
    println!("All valid words on this board:");
    let all_words = engine.find_all_valid_words(&board).await?;
    let mut sorted_words = all_words;
    sorted_words.sort_by_key(|a| std::cmp::Reverse(a.score()));

    for (i, word) in sorted_words.iter().take(20).enumerate() {
        println!("{}. {} (score: {})", i + 1, word.word, word.score());
    }
    if sorted_words.len() > 20 {
        println!("... and {} more words", sorted_words.len() - 20);
    }

    // Test different numbers of optimal words
    println!();
    println!("Testing optimal word finding for different n values:");
    for n in [1, 2, 3, 5, 10].iter() {
        let result = engine.find_best_n_words(&board, *n).await?;
        let (words, meta) = result;
        println!(
            "n={}: Found {} words, total score {}",
            n,
            words.len(),
            meta.total_score
        );
    }

    Ok(())
}
