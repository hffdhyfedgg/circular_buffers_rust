#ifndef DEBUG_H
#define DEBUG_H

#ifdef __cplusplus
extern "C" {
#endif

/* Конфигурация отладки - пользователь может переопределить перед включением */
#ifndef DEBUG_ENABLE
#define DEBUG_ENABLE 0
#endif

#ifndef DEBUG_SHOW_LINE
#define DEBUG_SHOW_LINE 0
#endif

#ifndef DEBUG_SHOW_LOCATION
#define DEBUG_SHOW_LOCATION 0
#endif

#ifndef DEBUG_DELIMETER
#define DEBUG_DELIMETER ALL
#endif

#ifndef ENABLE_COLOR
#define ENABLE_COLOR 0
#endif

/* Color configuration - can be redefined by user */
#if ENABLE_COLOR
#ifndef DEBUG_COLOR_FILE
#define DEBUG_COLOR_FILE "\x1b[36m" /* Cyan for filenames */
#endif

#ifndef DEBUG_COLOR_FUNC
#define DEBUG_COLOR_FUNC "\x1b[33m" /* Yellow for function names */
#endif

#ifndef DEBUG_COLOR_LINE
#define DEBUG_COLOR_LINE "\x1b[35m" /* Magenta for line numbers */
#endif

#ifndef DEBUG_COLOR_DELIM
#define DEBUG_COLOR_DELIM "\x1b[37m" /* White for delimiters */
#endif

#ifndef DEBUG_COLOR_RESET
#define DEBUG_COLOR_RESET "\x1b[0m" /* Reset to default */
#endif

#ifndef ERR_COLOR_DELIM
#define ERR_COLOR_DELIM "\x1b[31m" /* White for delimiters */
#endif
#else
/* When color is disabled, define all color macros as empty */
#define DEBUG_COLOR_FILE ""
#define DEBUG_COLOR_FUNC ""
#define DEBUG_COLOR_LINE ""
#define DEBUG_COLOR_DELIM ""
#define DEBUG_COLOR_RESET ""
#endif

/* Delimiter definitions */
#ifndef DEBUG_DELIMETER_MIN
#define DEBUG_DELIMETER_MIN "==================="
#endif
#ifndef DEBUG_DELIMETER_ALL
#define DEBUG_DELIMETER_ALL "--------------------"
#endif
#ifndef ERR_DELIMETER
#define ERR_DELIMETER       "********************"
#endif


#if DEBUG_ENABLE
/* Платформенно-зависимые заголовки */
#if defined(_WIN32) || defined(_WIN64)
#include <windows.h>
#endif

#include <stdarg.h>
#include <stdio.h>

/* Объявления функций */

void debug_stack_trace(void);
#endif

/* Основные отладочные макросы */
#if DEBUG_ENABLE

/* Внутренняя функция для обработки переменного числа аргументов */
static void _debug_printf(FILE *stream, const char *format, ...) {
    va_list args;
    va_start(args, format);
    vfprintf(stream, format, args);
    va_end(args);
    fflush(stream); /* Обеспечиваем немедленный вывод */
}

/* Внутренний макрос для префикса с информацией о расположении */
#if DEBUG_SHOW_LOCATION && DEBUG_SHOW_LINE
#define DEBUG_PREFIX(stream)                                                        \
    do {                                                                            \
        fprintf(stream, "[" DEBUG_COLOR_FILE "%s" DEBUG_COLOR_RESET ":", __FILE__); \
        fprintf(stream, DEBUG_COLOR_LINE "%d" DEBUG_COLOR_RESET "] ", __LINE__);    \
        fprintf(stream, DEBUG_COLOR_FUNC "%s" DEBUG_COLOR_RESET ": ", __func__);    \
    } while (0)
#elif DEBUG_SHOW_LOCATION
#define DEBUG_PREFIX(stream)                                                         \
    do {                                                                             \
        fprintf(stream, "[" DEBUG_COLOR_FILE "%s" DEBUG_COLOR_RESET "] ", __FILE__); \
        fprintf(stream, DEBUG_COLOR_FUNC "%s" DEBUG_COLOR_RESET ": ", __func__);     \
    } while (0)
#elif DEBUG_SHOW_LINE
#define DEBUG_PREFIX(stream)                                                         \
    do {                                                                             \
        fprintf(stream, "[" DEBUG_COLOR_LINE "%d" DEBUG_COLOR_RESET "] ", __LINE__); \
    } while (0)
#else
#define DEBUG_PREFIX(stream) ((void)0)
#endif

/* Макросы для вывода отладочной информации */  // fprintf(stderr, "\n" DEBUG_COLOR_DELIM "%s" DEBUG_COLOR_RESET "\n",
                                                // DEBUG_DELIMETER_ALL);
                                                //fprintf(stdout, DEBUG_COLOR_DELIM "%s" DEBUG_COLOR_RESET "\n", DEBUG_DELIMETER_ALL);
                                                //fprintf(stderr, ERR_COLOR_DELIM "%s" DEBUG_COLOR_RESET "\n", ERR_DELIMETER);
#define DEBUG_STDOUT(...)                                                                    \
    do {                                                                                     \
        DEBUG_PREFIX(stdout);                                                                \
        _debug_printf(stdout, __VA_ARGS__);                                                  \
        fprintf(stdout, "\n");                                                                       \
    } while (0)

#define DEBUG_STDERR(...)                                                                    \
    do {                                                                                     \
        DEBUG_PREFIX(stderr);                                                                \
        _debug_printf(stderr, __VA_ARGS__);                                                  \
        fprintf(stderr, "\n");                                                                       \
    } while (0)

/* Макрос для вывода текущего расположения */
#define DEBUG_LOCATION() DEBUG_STDOUT("\n")

/* Макрос для выполнения кода только в отладочном режиме */
#define DEBUG_CODE(...) \
    do {                \
        __VA_ARGS__     \
    } while (0)

/* Макрос для вывода стека вызовов */
#define DEBUG_STACK_TRACE() debug_stack_trace()

// Исключительно для отладки в python с cffi
void py_free(void* ptr);

#else /* DEBUG_ENABLE == 0 */

/* Все отладочные макросы становятся пустышками */
#define DEBUG_PREFIX(stream) ((void)0)
#define DEBUG_STDOUT(...) ((void)0)
#define DEBUG_STDERR(...) ((void)0)
#define DEBUG_LOCATION() ((void)0)
#define DEBUG_CODE(...) ((void)0)
#define DEBUG_STACK_TRACE() ((void)0)

#endif /* DEBUG_ENABLE */

#ifdef __cplusplus
}
#endif

#endif /* DEBUG_H */
