#ifndef DEMA_ERR_H
#define DEMA_ERR_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>

/**
 * @brief Тип для кодов ошибок библиотеки.
 *
 * Все коды ошибок определяются как отрицательные значения или ноль (успех).
 * Это позволяет использовать `int` как универсальный тип возвращаемого значения,
 * где положительные значения или 0 могут использоваться для результата операции,
 * а отрицательные - для кодов ошибок.
 */
typedef enum {
    DEMA_ERR_OK = 0,          ///< Операция выполнена успешно
    DEMA_ERR_NULL_POINTER = -1,    ///< Передан нулевой указатель
    DEMA_ERR_INVALID_ARG = -2,     ///< Передан недопустимый аргумент
    DEMA_ERR_MEM_ALLOC = -3,       ///< Ошибка выделения памяти
    DEMA_ERR_NOT_FOUND = -4,       ///< Элемент не найден
    DEMA_ERR_TIMEOUT = -5,         ///< Превышено время ожидания
    DEMA_ERR_IO_ERROR = -6,        ///< Ошибка ввода-вывода
    DEMA_ERR_NOT_SUPPORTED = -7,   ///< Операция не поддерживается
    DEMA_ERR_BUSY = -8,            ///< Ресурс занят
    DEMA_ERR_UNKNOWN  = -9         ///< Неизвестная ошибка.
    // Добавляйте новые коды ошибок здесь. DEMA_ERR_UNKNOWN всегда последняя в этом enum!
} dema_err_e;

#ifdef DEMA_ERR_ENABLE_MSG

/**
 * @brief Получает строковое описание ошибки.
 *
 * @param err Код ошибки типа dema_err_e.
 * @return Указатель на строку с описанием ошибки.
 *         Возвращаемая строка является статической и не должна освобождаться.
 */
const char* dema_err_get_msg(dema_err_e err);

#endif // DEMA_ERR_ENABLE_MSG

#ifdef __cplusplus
} // extern "C"
#endif

#endif // DEMA_ERR_H
