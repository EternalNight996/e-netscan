/// 打印颜色
fn output() {
    use e_utils::output;
    output!("hello world");
    output!(1;2;34; 5);
    let list = [1, 2, 34, 5];
    output!("{:#?}", list);
    output!(rgb[Some((0,255,0)), None] "打印自动义RGB");
}

fn main() {
    output();
}
    