class Point {
    x: number,
    y: number,
}

function main(): void {
    let p: Point = new Point();
    p.x = 10;
    p.y = 20;
    println("X: ");
    println(p.x);
    println("Y: ");
    println(p.y);
}
