#include "debug.h"
#include <stdio.h>
#include <stdarg.h>
#include <stdlib.h>
#include <math.h>

#if DEBUG_ENABLE
#include <stdlib.h>

// Исключительно для отладки в python с cffi
void py_free(void* ptr) {
  free(ptr);
}


#if !defined(_WIN32) && !defined(_WIN64) && !defined(__AVR__) && !defined(__PIC__)
#include <execinfo.h>
#endif

/* Внутренняя функция для обработки переменного числа аргументов */
// static void _debug_printf(FILE *stream, const char *format, ...) {
//     va_list args;
//     va_start(args, format);
//     vfprintf(stream, format, args);
//     va_end(args);
//     fflush(stream); /* Обеспечиваем немедленный вывод */
// }

/**
 * Инициализация отладочной среды
 * Настройка UTF-8 на Windows
 */
static void debug_init(void) {
#if defined(_WIN32) || defined(_WIN64)
    SetConsoleOutputCP(CP_UTF8);
#endif
}

/* Автоматическая инициализация для Windows */
#if defined(_WIN32) || defined(_WIN64)
#if defined(__GNUC__)
/* Для GCC/Clang используем конструктор */
__attribute__((constructor)) static void _debug_auto_init(void) {
    debug_init();
}
#elif defined(_MSC_VER)
/* Для MSVC используем инициализацию глобальной переменной */
#pragma section(".CRT$XCU",read)
static void __cdecl _debug_init_wrapper(void);
__declspec(allocate(".CRT$XCU")) void(*_debug_init_p)(void) = _debug_init_wrapper;
static void __cdecl _debug_init_wrapper(void) {
    debug_init();
}
#else
/* Для других компиляторов используем глобальную переменную */
static int _debug_init_var = (debug_init(), 0);
#endif
#else
/* Для не-Windows платформ инициализация не требуется */
#endif

/**
 * Вывод стека вызовов в читаемом виде
 */
void debug_stack_trace(void) {
#if !defined(_WIN32) && !defined(_WIN64) && !defined(__AVR__) && !defined(__PIC__)
    void *buffer[10];
    char **strings;
    int nptrs = backtrace(buffer, 10);

    strings = backtrace_symbols(buffer, nptrs);
    if (strings == NULL) {
        perror("backtrace_symbols");
        return;
    }

    DEBUG_STDERR("Stack trace:\n");
    for (int i = 0; i < nptrs; i++) {
        DEBUG_STDERR("  %s\n", strings[i]);
    }

    free(strings);
#else
    DEBUG_STDERR("Stack trace not implemented on this platform\n");
#endif
}
#endif /* DEBUG_ENABLE */
