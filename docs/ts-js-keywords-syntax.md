# TypeScript 与 JavaScript 关键词与语法大全

本文档整理了 TypeScript 和 JavaScript 的关键词、运算符、语句和语法特性，可作为 Nexa 编译器词法和语法分析的参考。

---

## 一、关键词总览

### 1.1 JavaScript 关键词（ES6+）

| 关键词 | 说明 | 示例 |
|--------|------|------|
| `var` | 函数作用域变量声明 | `var x = 1;` |
| `let` | 块级作用域变量声明 | `let x = 1;` |
| `const` | 块级作用域常量声明 | `const x = 1;` |
| `function` | 函数声明 | `function fn() {}` |
| `class` | 类声明 | `class Person {}` |
| `extends` | 类继承 | `class A extends B {}` |
| `super` | 调用父类 | `super();` |
| `this` | 当前上下文 | `this.name` |
| `return` | 函数返回值 | `return x;` |
| `if` | 条件语句 | `if (x) {}` |
| `else` | 条件否定分支 | `else {}` |
| `switch` | 多分支选择 | `switch(x) {}` |
| `case` | switch 分支 | `case 1: break;` |
| `default` | switch 默认分支 | `default: break;` |
| `for` | for 循环 | `for (let i=0; i<10; i++) {}` |
| `while` | while 循环 | `while (true) {}` |
| `do` | do-while 循环 | `do {} while(true);` |
| `break` | 跳出循环/switch | `break;` |
| `continue` | 跳过本次循环 | `continue;` |
| `try` | 异常处理尝试 | `try {}` |
| `catch` | 异常捕获 | `catch (e) {}` |
| `finally` | 无论如何都执行 | `finally {}` |
| `throw` | 抛出异常 | `throw new Error();` |
| `new` | 创建实例 | `new Date()` |
| `typeof` | 类型检查 | `typeof x` |
| `instanceof` | 实例检查 | `x instanceof Array` |
| `in` | 属性检查 | `'prop' in obj` |
| `delete` | 删除属性 | `delete obj.prop` |
| `void` | 返回 undefined | `void 0` |
| `with` | 扩展作用域（已废弃） | `with(obj) {}` |
| `debugger` | 调试断点 | `debugger;` |
| `async` | 异步函数声明 | `async function() {}` |
| `await` | 等待 Promise | `await fetch()` |
| `yield` | 生成器暂停 | `yield 1` |
| `export` | 导出模块 | `export default x` |
| `import` | 导入模块 | `import x from 'y'` |
| `null` | 空值字面量 | `null` |
| `true` | 布尔真值 | `true` |
| `false` | 布尔假值 | `false` |
| `enum` | 枚举声明 | `enum Color {}` |

### 1.2 TypeScript 特有关键词

| 关键词 | 说明 | 示例 |
|--------|------|------|
| `type` | 类型别名 | `type Num = number;` |
| `interface` | 接口定义 | `interface Person {}` |
| `implements` | 类实现接口 | `class A implements I {}` |
| `abstract` | 抽象类/方法 | `abstract class A {}` |
| `private` | 私有成员 | `private x: number;` |
| `protected` | 受保护成员 | `protected x: number;` |
| `public` | 公共成员 | `public x: number;` |
| `readonly` | 只读属性 | `readonly x: number;` |
| `static` | 静态成员 | `static x = 1;` |
| `declare` | 声明全局 | `declare var x: any;` |
| `namespace` | 命名空间 | `namespace NS {}` |
| `module` | 模块声明 | `module 'x' {}` |
| `any` | 任意类型 | `let x: any;` |
| `unknown` | 未知类型 | `let x: unknown;` |
| `never` | 从不返回值 | `function err(): never {}` |
| `as` | 类型断言 | `x as string` |
| `is` | 类型谓词 | `isString(x): x is string` |
| `keyof` | 键of类型 | `keyof T` |
| `infer` | 推断类型 | `infer R` |
| `asserts` | 断言函数 | `asserts x is string` |
| `override` | 方法重写 | `override fn() {}` |
| `satisfies` | 满足类型 | `x satisfies T` |
| `out` | 协变标记 | `out T` |
| `get` | getter | `get x() {}` |
| `set` | setter | `set x(v) {}` |
| `constructor` | 构造函数 | `constructor() {}` |
| `require` | CommonJS 导入 | `require('x')` |
| `unique` | 唯一符号 | `unique symbol` |

### 1.3 上下文关键词（特定场景可用作标识符）

| 关键词 | 可用作标识符 | 用法 |
|--------|-------------|------|
| `await` | 否（严格模式） | `await promise` |
| `yield` | 否（严格模式） | `yield* gen` |
| `implements` | 是 | 类实现接口 |
| `interface` | 是 | 定义接口 |
| `private` | 是 | 私有成员 |
| `protected` | 是 | 受保护成员 |
| `public` | 是 | 公共成员 |
| `static` | 是 | 静态成员 |
| `abstract` | 是 | 抽象成员 |
| `as` | 是 | 类型断言 |
| `is` | 是 | 类型谓词 |
| `keyof` | 是 | 键类型 |
| `infer` | 是 | 类型推断 |
| `out` | 是 | 协变泛型 |
| `override` | 是 | 重写方法 |
| `satisfies` | 是 | 类型满足 |
| `readonly` | 是 | 只读属性 |
| `module` | 是 | 模块声明 |
| `namespace` | 是 | 命名空间 |
| `declare` | 是 | 声明 |
| `get` | 是 | getter |
| `set` | 是 | setter |

### 1.4 内置类型

| 类型 | 说明 |
|------|------|
| `boolean` | 布尔值 `true` / `false` |
| `number` | 数字（64位浮点） |
| `string` | 字符串 |
| `bigint` | 大整数 |
| `symbol` | 唯一符号 |
| `object` | 对象类型 |
| `undefined` | 未定义 |
| `null` | 空值 |
| `void` | 无返回值 |
| `never` | 永不返回 |
| `unknown` | 未知类型 |
| `any` | 任意类型 |

---

## 二、运算符

### 2.1 算术运算符

| 运算符 | 说明 | 示例 |
|--------|------|------|
| `+` | 加法 / 字符串拼接 | `1 + 2` / `"a" + "b"` |
| `-` | 减法 | `5 - 3` |
| `*` | 乘法 | `2 * 3` |
| `/` | 除法 | `10 / 2` |
| `%` | 取余 | `10 % 3` |
| `**` | 幂运算 | `2 ** 3` |
| `++` | 递增 | `++x` / `x++` |
| `--` | 递减 | `--x` / `x--` |
| `+` | 正号 | `+5` |
| `-` | 负号 | `-5` |

### 2.2 赋值运算符

| 运算符 | 说明 | 等价于 |
|--------|------|--------|
| `=` | 赋值 | `x = 1` |
| `+=` | 加并赋值 | `x += 1` |
| `-=` | 减并赋值 | `x -= 1` |
| `*=` | 乘并赋值 | `x *= 2` |
| `/=` | 除并赋值 | `x /= 2` |
| `%=` | 取余并赋值 | `x %= 3` |
| `**=` | 幂并赋值 | `x **= 2` |
| `<<=` | 左移并赋值 | `x <<= 1` |
| `>>=` | 右移并赋值 | `x >>= 1` |
| `>>>=` | 无符号右移并赋值 | `x >>>= 1` |
| `&=` | 按位与并赋值 | `x &= 1` |
| `^=` | 按位异或并赋值 | `x ^= 1` |
| `\|=` | 按位或并赋值 | `x \|= 1` |
| `&&=` | 逻辑与并赋值 | `x &&= y` |
| `\|\|=` | 逻辑或并赋值 | `x \|\|= y` |
| `??=` | 空值合并并赋值 | `x ??= y` |

### 2.3 比较运算符

| 运算符 | 说明 | 特点 |
|--------|------|------|
| `==` | 相等 | 自动类型转换 |
| `!=` | 不等 | 自动类型转换 |
| `===` | 严格相等 | 不转换类型 |
| `!==` | 严格不等 | 不转换类型 |
| `>` | 大于 | |
| `<` | 小于 | |
| `>=` | 大于等于 | |
| `<=` | 小于等于 | |

### 2.4 逻辑运算符

| 运算符 | 说明 | 短路 |
|--------|------|------|
| `&&` | 逻辑与 | 是 |
| `\|\|` | 逻辑或 | 是 |
| `!` | 逻辑非 | - |
| `??` | 空值合并 | 是 |
| `?.` | 可选链 | 是 |
| `?.[` | 可选索引 | 是 |

### 2.5 位运算符

| 运算符 | 说明 |
|--------|------|
| `&` | 按位与 |
| `\|` | 按位或 |
| `^` | 按位异或 |
| `~` | 按位非 |
| `<<` | 左移 |
| `>>` | 右移（有符号） |
| `>>>` | 右移（无符号） |

### 2.6 一元运算符

| 运算符 | 说明 |
|--------|------|
| `typeof` | 返回类型字符串 |
| `delete` | 删除对象属性 |
| `void` | 返回 undefined |
| `await` | 等待 Promise |
| `+` | 转为数字 |
| `-` | 转为负数 |

### 2.7 三元运算符

```javascript
condition ? exprIfTrue : exprIfFalse
```

### 2.8 扩展运算符

| 运算符 | 说明 |
|--------|------|
| `...` | 展开数组/对象 |
| `...` | rest 参数 |

### 2.9 TypeScript 类型运算符

| 运算符 | 说明 | 示例 |
|--------|------|------|
| `\|` | 联合类型 | `string \| number` |
| `&` | 交叉类型 | `A & B` |
| `extends` | 泛型约束 | `T extends U` |
| `keyof` | 键类型 | `keyof T` |
| `infer` | 类型推断 | `infer R` |
| `as` | 类型断言 | `x as T` |
| `<T>` | 类型断言 | `<T>x` |
| `satisfies` | 类型满足 | `x satisfies T` |
| `is` | 类型谓词 | `x is T` |

---

## 三、语句

### 3.1 条件语句

```javascript
// if-else
if (condition) {
  // true
} else if (condition2) {
  // condition2 true
} else {
  // false
}

// 三元运算符
const x = condition ? trueValue : falseValue;

// switch
switch (value) {
  case 1:
    // ...
    break;
  case 2:
    // ...
    break;
  default:
    // ...
}
```

### 3.2 循环语句

```javascript
// for
for (let i = 0; i < 10; i++) {
  // ...
}

// for...in (遍历键)
for (const key in obj) {
  // ...
}

// for...of (遍历值)
for (const item of arr) {
  // ...
}

// forEach
arr.forEach((item, index) => {
  // ...
});

// while
while (condition) {
  // ...
}

// do...while
do {
  // ...
} while (condition);
```

### 3.3 异常处理

```javascript
try {
  // 可能抛出异常的代码
} catch (error) {
  // 捕获异常
  console.error(error);
} finally {
  // 总是执行
}

// 抛出异常
throw new Error('message');

// try-catch-finally 嵌套
try {
  try {
    throw new Error('inner');
  } catch (e) {
    throw new Error('outer');
  } finally {
    console.log('finally');
  }
} catch (e) {
  console.error(e);
}
```

### 3.4 函数声明

```javascript
// 函数声明
function greet(name) {
  return `Hello, ${name}`;
}

// 函数表达式
const greet = function(name) {
  return `Hello, ${name}`;
};

// 箭头函数
const greet = (name) => `Hello, ${name}`;
const greet = (name) => {
  return `Hello, ${name}`;
};

// 构造函数（不推荐）
const Person = new Function('name', 'this.name = name');

// 异步函数
async function fetchData() {
  const response = await fetch(url);
  return response.json();
}

// 生成器
function* gen() {
  yield 1;
  yield 2;
}

// 类方法
class Person {
  constructor(name) {
    this.name = name;
  }
  
  greet() {
    return `Hello, ${this.name}`;
  }
  
  static create(name) {
    return new Person(name);
  }
  
  get fullName() {
    return this.name;
  }
  
  set fullName(value) {
    this.name = value;
  }
}
```

### 3.5 类声明

```javascript
// 基本类
class Animal {
  // 构造函数
  constructor(name) {
    this.name = name;
  }
  
  // 实例方法
  speak() {
    console.log(`${this.name} makes a noise.`);
  }
  
  // 静态方法
  static create(name) {
    return new Animal(name);
  }
}

// 继承
class Dog extends Animal {
  constructor(name, breed) {
    super(name); // 调用父类构造函数
    this.breed = breed;
  }
  
  // 重写方法
  speak() {
    console.log(`${this.name} barks!`);
  }
}

// 接口实现（TypeScript）
interface Printable {
  print(): void;
}

class Document implements Printable {
  print() {
    console.log('Printing...');
  }
}

// 抽象类（TypeScript）
abstract class Shape {
  abstract area(): number;
  
  describe() {
    return `Area: ${this.area()}`;
  }
}
```

### 3.6 模块语法

```javascript
// 导出
export const PI = 3.14;
export function add(a, b) { return a + b; }
export class Calculator { }
export default function main() { }
export { PI as PI_VALUE };
export * from './module';

// 导入
import { PI, add } from './module';
import * as Utils from './module';
import main from './module';
import { PI as PI_VALUE } from './module';
```

### 3.7 TypeScript 类型声明

```typescript
// 类型别名
type ID = string | number;
type Callback = () => void;
type Pair<T> = { key: T; value: T };

// 接口
interface User {
  id: number;
  name: string;
  email?: string;      // 可选属性
  readonly createdAt: Date;  // 只读
}

interface Admin extends User {
  role: 'admin' | 'superadmin';
}

// 枚举
enum Color {
  Red = 'red',
  Green = 'green',
  Blue = 'blue'
}

// 泛型
function identity<T>(arg: T): T {
  return arg;
}

interface Container<T> {
  value: T;
}

class Box<T> {
  constructor(public value: T) {}
}

// 交叉类型
type Extended = User & { role: string };

// 联合类型
type Status = 'pending' | 'success' | 'error';
```

---

## 四、数据类型

### 4.1 原始类型

```javascript
// 字符串
const str = "Hello";
const template = `Hello, ${name}`;

// 数字
const num = 42;
const float = 3.14;
const hex = 0xFF;
const binary = 0b1010;
const octal = 0o755;

// 布尔
const bool = true;
const bool2 = false;

// BigInt
const big = 9007199254740991n;

// Symbol
const sym = Symbol('description');
const unique = Symbol();

// undefined
let undef;
const n = undefined;

// null
const n = null;
```

### 4.2 引用类型

```javascript
// 对象
const obj = {
  name: 'John',
  age: 30,
  greet() {
    return `Hello, ${this.name}`;
  }
};

// 数组
const arr = [1, 2, 3];
const mixed = [1, 'two', true];

// Map
const map = new Map();
map.set('key', 'value');

// Set
const set = new Set([1, 2, 3]);

// WeakMap / WeakSet
const weakMap = new WeakMap();
const weakSet = new WeakSet();

// Date
const date = new Date();

// RegExp
const regex = /pattern/flags;

// Error
const error = new Error('message');

// Promise
const promise = new Promise((resolve, reject) => {
  // ...
});
```

---

## 五、语法特性演进

### ES5 (2009)
- `var` 声明
- `function` 表达式
- 原型继承
- `JSON.parse/stringify`
- `Array.prototype.map/filter/reduce`

### ES6 / ES2015
- `let` / `const`
- 箭头函数
- 类语法
- 模块导入/导出 (`import`/`export`)
- 模板字符串
- 解构赋值
- 扩展运算符
- Promise
- 生成器
- Symbol
- `for...of`
- 默认参数
- 剩余参数

### ES7 / ES2016
- 指数运算符 (`**`)
- `Array.prototype.includes`

### ES8 / ES2017
- `async`/`await`
- `Object.entries/values`
- 尾随逗号

### ES9 / ES2018
- 异步迭代 (`for await...of`)
- 展开运算符（对象）
- 正则表达式增强

### ES10 / ES2019
- `Array.prototype.flat/flatMap`
- `Object.fromEntries`
- 可选 catch 绑定
- `String.prototype.trimStart/trimEnd`

### ES11 / ES2020
- 可选链 (`?.`)
- 空值合并 (`??`)
- `BigInt`
- `Promise.allSettled`
- `dynamic import`

### ES12 / ES2021
- 逻辑赋值运算符 (`&&=`/`||=`/`??=`)
- `String.prototype.replaceAll`
- 数字分隔符 (`1_000_000`)
- Promise.any

### ES13 / ES2022
- 类字段声明
- 静态块
- `#private` 私有字段
- `at()` 方法
- 正则表达式 `d` 标志
- Top-level await

### ES14 / ES2023
- `Array.prototype.toSorted/toReversed/toSpliced/with`
- `Symbol.dispose`
- `String.prototype.isWellFormed/toWellFormed`
- 哈希bang语法

---

## 六、TypeScript 特有语法

### 6.1 类型注解

```typescript
// 变量类型
let num: number = 42;
let str: string = 'hello';
let arr: number[] = [1, 2, 3];
let tuple: [string, number] = ['hello', 42];

// 函数类型
function greet(name: string): string {
  return `Hello, ${name}`;
}

const greet = (name: string): string => `Hello, ${name}`;

// 对象类型
interface User {
  name: string;
  age: number;
  email?: string;  // 可选
  readonly id: number;  // 只读
}

type Config = {
  theme: 'light' | 'dark';
  [key: string]: any;  // 索引签名
};
```

### 6.2 泛型

```typescript
// 泛型函数
function identity<T>(arg: T): T {
  return arg;
}

// 泛型接口
interface Container<T> {
  value: T;
  get(): T;
}

// 泛型类
class Box<T> {
  constructor(private value: T) {}
}

// 泛型约束
interface Lengthwise {
  length: number;
}

function logLength<T extends Lengthwise>(arg: T): number {
  return arg.length;
}

// 多类型参数
function pair<K, V>(k: K, v: V): [K, V] {
  return [k, v];
}

// 默认类型
function create<T = string>(value?: T): T {
  return value || (null as any);
}
```

### 6.3 条件类型

```typescript
// 基础条件类型
type IsString<T> = T extends string ? true : false;

// 提取返回类型
type ReturnType<T> = T extends (...args: any[]) => infer R ? R : never;

// 提取参数类型
type Parameters<T> = T extends (...args: infer P) => any ? P : never;

// 映射类型
type Readonly<T> = {
  readonly [P in keyof T]: T[P];
};

type Partial<T> = {
  [P in keyof T]?: T[P];
};

type Pick<T, K extends keyof T> = {
  [P in K]: T[P];
};

type Omit<T, K extends keyof T> = {
  [P in Exclude<keyof T, K>]: T[P];
};
```

### 6.4 装饰器 (实验性)

```typescript
// 类装饰器
function logged(target: Function) {
  console.log('Class called');
  return target;
}

// 方法装饰器
function enumerable(value: boolean) {
  return function (
    target: any,
    propertyKey: string,
    descriptor: PropertyDescriptor
  ) {
    descriptor.enumerable = value;
  };
}

// 访问器装饰器
function configurable(value: boolean) {
  return function (
    target: any,
    propertyKey: string,
    descriptor: PropertyDescriptor
  ) {
    descriptor.configurable = value;
  };
}

// 参数装饰器
function param(target: any, propertyKey: string, parameterIndex: number) {
  // ...
}

// 使用
@logged
class MyClass {
  @enumerable(false)
  method() {}
  
  @configurable(false)
  get value() { return 1; }
  
  method(@param value: string) {}
}
```

---

## 七、速查表

### 7.1 常用声明对比

| 特性 | `var` | `let` | `const` |
|------|-------|-------|---------|
| 作用域 | 函数 | 块 | 块 |
| 提升 | 是 | 暂存死区 | 暂存死区 |
| 重复声明 | 允许 | 不允许 | 不允许 |
| 可修改 | 是 | 是 | 引用不可改 |
| 初始值 | 可选 | 可选 | 必须 |

### 7.2 类型检查对比

| 运算符/方法 | 说明 | 示例 |
|-------------|------|------|
| `typeof` | 原始类型 | `typeof 'str'` → `'string'` |
| `instanceof` | 实例检查 | `arr instanceof Array` |
| `in` | 属性存在 | `'toString' in obj` |
| `Array.isArray()` | 数组检查 | `Array.isArray([])` |
| `Object.prototype.hasOwnProperty()` | 自有属性 | `obj.hasOwnProperty('key')` |

### 7.3 常用简写

```javascript
// 解构赋值
const { name, age } = person;
const [first, ...rest] = array;
const { name: userName } = person;

// 展开
const merged = { ...obj1, ...obj2 };
const combined = [...arr1, ...arr2];

// 可选链
const street = user?.address?.street;

// 空值合并
const value = data ?? 'default';

// 逻辑赋值
config ??= defaultConfig;
user.name &&= 'Guest';

// 数字分隔符
const billion = 1_000_000_000;
```

---

## 八、分词器 Token 分类参考

Nexa 编译器词法分析器可将 Token 分为以下类别：

### 8.1 关键字 Token

```
KEYWORD_IF, KEYWORD_ELSE, KEYWORD_FOR, KEYWORD_WHILE, KEYWORD_DO,
KEYWORD_SWITCH, KEYWORD_CASE, KEYWORD_DEFAULT, KEYWORD_BREAK, KEYWORD_CONTINUE,
KEYWORD_RETURN, KEYWORD_THROW, KEYWORD_TRY, KEYWORD_CATCH, KEYWORD_FINALLY,
KEYWORD_CLASS, KEYWORD_EXTENDS, KEYWORD_SUPER, KEYWORD_NEW, KEYWORD_THIS,
KEYWORD_FUNCTION, KEYWORD_ARROW, KEYWORD_ASYNC, KEYWORD_AWAIT, KEYWORD_YIELD,
KEYWORD_VAR, KEYWORD_LET, KEYWORD_CONST,
KEYWORD_IMPORT, KEYWORD_EXPORT, KEYWORD_FROM,
KEYWORD_TYPE, KEYWORD_INTERFACE, KEYWORD_ENUM, KEYWORD_NAMESPACE, KEYWORD_MODULE,
KEYWORD_PUBLIC, KEYWORD_PRIVATE, KEYWORD_PROTECTED, KEYWORD_STATIC, KEYWORD_READONLY,
KEYWORD_ABSTRACT, KEYWORD_IMPLEMENTS, KEYWORD_DECLARE,
KEYWORD_ANY, KEYWORD_UNKNOWN, KEYWORD_NEVER, KEYWORD_VOID,
KEYWORD_TYPEOF, KEYWORD_INSTANCEOF, KEYWORD_IN, KEYWORD_DELETE,
KEYWORD_TRUE, KEYWORD_FALSE, KEYWORD_NULL, KEYWORD_UNDEFINED,
KEYWORD_DEBUGGER, KEYWORD_WITH
```

### 8.2 运算符 Token

```
OPERATOR_PLUS, OPERATOR_MINUS, OPERATOR_MULTIPLY, OPERATOR_DIVIDE, OPERATOR_MODULO,
OPERATOR_EXPONENT, OPERATOR_INCREMENT, OPERATOR_DECREMENT,
OPERATOR_ASSIGN, OPERATOR_PLUS_ASSIGN, OPERATOR_MINUS_ASSIGN, etc.
OPERATOR_EQUAL, OPERATOR_STRICT_EQUAL, OPERATOR_NOT_EQUAL, OPERATOR_STRICT_NOT_EQUAL,
OPERATOR_GREATER, OPERATOR_LESS, OPERATOR_GREATER_EQUAL, OPERATOR_LESS_EQUAL,
OPERATOR_LOGICAL_AND, OPERATOR_LOGICAL_OR, OPERATOR_LOGICAL_NOT,
OPERATOR_BITWISE_AND, OPERATOR_BITWISE_OR, OPERATOR_BITWISE_XOR, OPERATOR_BITWISE_NOT,
OPERATOR_BITWISE_LEFT_SHIFT, OPERATOR_BITWISE_RIGHT_SHIFT, OPERATOR_BITWISE_UNSIGNED_RIGHT_SHIFT,
OPERATOR_OPTIONAL_CHAIN, OPERATOR_NULLISH_COALESCING,
OPERATOR_SPREAD, OPERATOR_REST,
OPERATOR_QUESTION, OPERATOR_COLON, // 三元运算符
OPERATOR_DOT, OPERATOR_COMMA, OPERATOR_SEMICOLON,
OPERATOR_OPEN_PAREN, OPERATOR_CLOSE_PAREN,
OPERATOR_OPEN_BRACE, OPERATOR_CLOSE_BRACE,
OPERATOR_OPEN_BRACKET, OPERATOR_CLOSE_BRACKET
```

### 8.3 字面量 Token

```
LITERAL_NUMBER, LITERAL_BIGINT, LITERAL_STRING, LITERAL_TEMPLATE,
LITERAL_REGEX, LITERAL_IDENTIFIER,
LITERAL_TRUE, LITERAL_FALSE, LITERAL_NULL, LITERAL_UNDEFINED
```

### 8.4 类型相关 Token（TypeScript）

```
TYPE_OPERATOR_UNION, TYPE_OPERATOR_INTERSECTION,
TYPE_OPERATOR_KEYOF, TYPE_OPERATOR_INFER, TYPE_OPERATOR_EXTENDS,
TYPE_ASSERTION_AS, TYPE_ASSERTION_ANGLE_BRACKET,
TYPE_ANNOTATION_COLON, TYPE_GENERIC_LESS, TYPE_GENERIC_GREAT
```

---

> 文档版本：1.0
> 最后更新：2026-03-18
