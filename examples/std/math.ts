// Nexa 标准库 - math 模块示例
// 演示 import/export 语法

// 导出 math 模块的函数
export function add(a: number, b: number): number {
    return a + b;
}

export function subtract(a: number, b: number): number {
    return a - b;
}

export function multiply(a: number, b: number): number {
    return a * b;
}

export function divide(a: number, b: number): number {
    if (b == 0) {
        return 0;
    }
    return a / b;
}

export const PI: number = 3.14159;
export const E: number = 2.71828;
