// Nexa 标准库 - main.ts 示例
// 演示如何使用 import 语法

// 导入 io 模块
import { println, print, readln, VERSION } from "std/io";

// 导入 math 模块（使用命名空间）
import * as math from "std/math";

function main(): void {
    println("Welcome to Nexa Standard Library Demo!");
    println("Version: " + VERSION);
    
    // 使用 math 模块的函数
    let a: number = 10;
    let b: number = 5;
    
    println("a = " + a);
    println("b = " + b);
    println("a + b = " + math.add(a, b));
    println("a - b = " + math.subtract(a, b));
    println("a * b = " + math.multiply(a, b));
    println("a / b = " + math.divide(a, b));
    println("PI = " + math.PI);
    
    // 演示输入功能
    println("Enter a number:");
    let input: number = readln();
    println("You entered: " + input);
}
