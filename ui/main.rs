fn main() {
    let data = vec![1, 2, 3];
    let x: Vec<u32> = data.into_iter().collect();
    println!("{:?}", x);

    let data2 = vec![1, 2, 3];
    let y = data2.into_iter().collect::<Vec<u32>>();
    println!("{:?}", y);
    let data3 = vec![1, 2, 3];
    let mut z: Vec<u32> = data3.into_iter().collect();
    z.push(4);
    println!("{:?}", z);
}
