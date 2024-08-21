use gufo_jpeg::Jpeg;

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("First agument must be a file path.");
    let data = std::fs::read(path).unwrap();
    let jpeg = Jpeg::new(data);

    for segment in jpeg.segments() {
        let data_init = segment
            .data()
            .iter()
            .take(50)
            .take_while(|x| **x != b'\0')
            .filter(|x| x.is_ascii_graphic())
            .cloned()
            .collect::<Vec<_>>();
        let s = String::from_utf8_lossy(&data_init);

        println!("{:x?}: {s}", segment.marker());
    }
}
