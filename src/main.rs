mod d3d12;

fn main() {
    let game = d3d12::BareBoneGame::new(String::from("샘플"), 1280, 720).unwrap();
}
