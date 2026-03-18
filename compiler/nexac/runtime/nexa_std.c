// Nexa 标准库运行时实现
// 提供 std.io.* 函数的 C 实现

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdarg.h>

// ============ 基础 IO 函数 ============

// std.io.println - 打印字符串并换行
int std_io_println(const char* s) {
    return printf("%s\n", s);
}

// std.io.print - 打印字符串不换行
int std_io_print(const char* s) {
    return printf("%s", s);
}

// std.io.readln - 读取一行输入 (返回动态分配的字符串)
char* std_io_readln() {
    char* line = NULL;
    size_t len = 0;
    ssize_t nread = getline(&line, &len, stdin);
    if (nread == -1) {
        free(line);
        return NULL;
    }
    // 移除换行符
    if (nread > 0 && line[nread - 1] == '\n') {
        line[nread - 1] = '\0';
    }
    return line;
}

// ============ 数字打印函数 ============

// std.io.println_i32 - 打印整数并换行
int std_io_println_i32(int n) {
    return printf("%d\n", n);
}

// std.io.print_i32 - 打印整数不换行
int std_io_print_i32(int n) {
    return printf("%d", n);
}

// ============ 文件 IO 函数 ============

// 文件打开 - 返回文件指针 (FILE*)
void* std_io_fopen(const char* filename, const char* mode) {
    FILE* f = fopen(filename, mode);
    return (void*)f;
}

// 文件关闭
int std_io_fclose(void* file) {
    return fclose((FILE*)file);
}

// 读取一行 - 返回动态分配的字符串
char* std_io_fgets(char* buffer, int size, void* file) {
    return fgets(buffer, size, (FILE*)file);
}

// 写入字符串
int std_io_fputs(const char* str, void* file) {
    return fputs(str, (FILE*)file);
}

// 写入格式化字符串
int std_io_fprintf(void* file, const char* format, ...) {
    va_list args;
    va_start(args, format);
    int result = vfprintf((FILE*)file, format, args);
    va_end(args);
    return result;
}

// 检查文件是否结束
int std_io_feof(void* file) {
    return feof((FILE*)file);
}

// 检查是否有错误
int std_io_ferror(void* file) {
    return ferror((FILE*)file);
}

// ============ 字符串函数 ============

// 字符串长度
int std_string_len(const char* str) {
    return strlen(str);
}

// 字符串复制
char* std_string_copy(const char* src) {
    return strdup(src);
}

// 字符串比较
int std_string_compare(const char* s1, const char* s2) {
    return strcmp(s1, s2);
}

// 字符串拼接
char* std_string_concat(const char* s1, const char* s2) {
    size_t len1 = strlen(s1);
    size_t len2 = strlen(s2);
    char* result = malloc(len1 + len2 + 1);
    strcpy(result, s1);
    strcat(result, s2);
    return result;
}

// 字符串转整数
int std_string_to_i32(const char* str) {
    return atoi(str);
}

// 整数转字符串
char* std_string_from_i32(int n) {
    char* buffer = malloc(32);
    snprintf(buffer, 32, "%d", n);
    return buffer;
}

// ============ 内存管理函数 ============

// 分配内存
void* std_memory_alloc(size_t size) {
    return malloc(size);
}

// 释放内存
void std_memory_free(void* ptr) {
    free(ptr);
}

// 重新分配内存
void* std_memory_realloc(void* ptr, size_t new_size) {
    return realloc(ptr, new_size);
}

// 内存复制
void* std_memory_copy(void* dest, const void* src, size_t n) {
    return memcpy(dest, src, n);
}

// 内存移动 (处理重叠区域)
void* std_memory_move(void* dest, const void* src, size_t n) {
    return memmove(dest, src, n);
}

// 内存设置
void* std_memory_set(void* s, int c, size_t n) {
    return memset(s, c, n);
}

// 内存比较
int std_memory_compare(const void* s1, const void* s2, size_t n) {
    return memcmp(s1, s2, n);
}

// ============ 数学函数 ============

// 绝对值
int std_math_abs(int n) {
    return n < 0 ? -n : n;
}

// 最大值
int std_math_max(int a, int b) {
    return a > b ? a : b;
}

// 最小值
int std_math_min(int a, int b) {
    return a < b ? a : b;
}

// 幂运算
double std_math_pow(double base, double exp) {
    return pow(base, exp);
}

// 平方根
double std_math_sqrt(double n) {
    return sqrt(n);
}
