#[derive(Debug)]
struct Rectangle {
    width: u32,
    height: u32,
}

impl Rectangle {
    fn area(&self) -> u32 {
        self.width * self.height
    }
    fn can_hold(&self, other: &Rectangle) -> bool {
        self.width >= other.width && self.height >= other.height
    }
    fn square(size: u32) -> Rectangle {
        Rectangle {
            width: size,
            height: size,
        }
    }
}

use std::io;

fn read_input<T: std::str::FromStr>(prompt: &str) -> T
where
    T::Err: std::fmt::Debug,
{
    let mut buffer = String::new();
    println!("{}", prompt);
    io::stdin().read_line(&mut buffer).expect("Failed to read");
    buffer.trim().parse().expect("Non-numeric input")
}

fn main() {
    fn read_rect() -> Rectangle {
        Rectangle {
            width: read_input("Enter width:"),
            height: read_input("Enter height:"),
        }
    }
    println!("== Rect 1 ==");
    let rect1 = dbg!(read_rect());
    println!("Area: {}", rect1.area());

    println!("== Rect 2 ==");
    let rect2 = dbg!(read_rect());
    println!("Area: {}", rect2.area());

    println!("== Square ==");
    let square = dbg!(Rectangle::square(read_input("Enter size:")));
    println!("Area: {}", square.area());

    println!("Rect 1 can hold Rect 2: {}", rect1.can_hold(&rect2));
    println!("Rect 1 can hold Square: {}", rect1.can_hold(&square));
}
