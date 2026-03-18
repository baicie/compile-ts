class Point {
    x: number,
    y: number,
    
    // 构造函数
    constructor(x: number, y: number) {
        this.x = x;
        this.y = y;
    }
    
    // 方法
    getX(): number {
        return this.x;
    }
    
    getY(): number {
        return this.y;
    }
}

function main(): void {
    let p: Point = new Point(10, 20);
    println("X: ");
    println(p.getX());
    println("Y: ");
    println(p.getY());
}
