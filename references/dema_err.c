#include "dema_err.h"

// Включаем реализацию только если флаг установлен
#ifdef DEMA_ERR_ENABLE_MSG
#include <stdlib.h>
// Массив строк с описаниями ошибок.
// Индексы в массиве соответствуют значениям перечисления dema_err_e.
// ВАЖНО: Порядок строк должен точно соответствовать порядку в enum.
static const char * const dema_err_msgs[] = {
    [DEMA_ERR_OK]             = "ru: Завершено без ошибок.\nen: Completed without errors.",
    [abs(DEMA_ERR_NULL_POINTER)]   = "ru: Передан NULL указатель.\nen: NULL pointer passed.",
    [abs(DEMA_ERR_INVALID_ARG)]    = "ru: Неверный аргумент.\nen: Invalid argument.",
    [abs(DEMA_ERR_MEM_ALLOC)]      = "ru: Недостаточно памяти.\nen: Out of memory.",
    [abs(DEMA_ERR_NOT_FOUND)]      = "ru: Элемент не найден.\nen: Item not found.",
    [abs(DEMA_ERR_TIMEOUT)]        = "ru: Операция завершилась по таймауту.\nen: Operation timed out.",
    [abs(DEMA_ERR_IO_ERROR)]       = "ru: Ошибка ввода/вывода.\nen: Input/Output error.",
    [abs(DEMA_ERR_NOT_SUPPORTED)]  = "ru: Операция не поддерживается.\nen: Operation not supported.",
    [abs(DEMA_ERR_BUSY)]           = "ru: Ресурс занят.\nen: Resource is busy.",
    [abs(DEMA_ERR_UNKNOWN)]        = "ru: Неизвестная ошибка\nen: Unknown error."
    // Добавляйте новые описания здесь, в том же порядке
};

const char* dema_err_get_msg(dema_err_e err) {
    // Проверяем, что код ошибки в допустимом диапазоне
    if (err <= 0 && err >= DEMA_ERR_UNKNOWN) {
        const char* msg = dema_err_msgs[err];
        return msg; // Возвращаем описание, если оно определено
    }
    // Если код ошибки не распознан или описание отсутствует
    return dema_err_msgs[dema_err_e];
}

#endif // DEMA_ERR_ENABLE_MSG
