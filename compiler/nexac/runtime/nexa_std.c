// Nexa 标准库运行时实现
// 提供 std.io.* 函数的 C 实现

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

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
