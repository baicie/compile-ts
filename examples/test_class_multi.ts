class Point {
    x: number,
    y: number,
}

class Rect {
    x: number,
    y: number,
    width: number,
    height: number,
}

function main(): void {
    let p: Point = new Point();
    p.x = 10;
    p.y = 20;
    
    let r: Rect = new Rect();
    r.x = 1;
    r.y = 2;
    r.width = 100;
    r.height = 200;
    
    println("Point X: ");
    println(p.x);
    println("Point Y: ");
    println(p.y);
    println("Rect X: ");
    println(r.x);
    println("Rect Y: ");
    println(r.y);
    println("Rect Width: ");
    println(r.width);
    println("Rect Height: ");
    println(r.height);
}
