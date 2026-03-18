// Nexa 标准库 - main 示例
// 演示如何使用 std 包

// 使用命名空间导入
import * as io from "std/io";

function main(): void {
    io.println("Welcome to Nexa!");
    io.println("This is a test of the std package.");
    
    let x: number = 42;
    io.println("Value of x: ");
    // 注意：目前不支持数字和字符串的直接拼接
    // io.println("Value of x: " + x);  // 这行会编译错误
}
