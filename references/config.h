#include "float.h"
#include <math.h>

#ifndef _flt
#define _flt DBL // Разрешено
#endif

#if _flt == FLT

#define _flt_type float
#define _FLT_MIN FLT_MIN
#define _FLT_MAX FLT_MAX
#define _FLT_N_MIN -FLT_MAX
#define _FLT_N_MAX -FLT_MIN
#define _FLT_ABS(value) fabsf((value))

#elif _flt == DBL

#define _flt_type double
#define _FLT_MIN DBL_MIN
#define _FLT_MAX DBL_MAX
#define _FLT_N_MIN -DBL_MAX
#define _FLT_N_MAX -DBL_MIN
#define _FLT_ABS(value) fabs((value))

#elif _flt == LDBL

#define _flt_type long double
#define _FLT_MIN LDBL_MIN
#define _FLT_MAX LDBL_MAX
#define _FLT_N_MIN -LDBL_MAX
#define _FLT_N_MAX -LDBL_MIN
#define _FLT_ABS(value) fabsl((value))

#else

#error "Выбран недопустимый тип с плавающей точкой!"

#endif

#if defined(_MSC_VER)  /* Visual Studio: безопасные функции доступны всегда */
    #define PRINTF_S(...) printf_s(__VA_ARGS__)
#elif defined(__STDC_LIB_EXT1__) && defined(__STDC_WANT_LIB_EXT1__) && __STDC_WANT_LIB_EXT1__ == 1
    /* Реализация поддерживает Annex K, и пользователь запросил расширения */
    #define PRINTF_S(...) printf_s(__VA_ARGS__)
#else
    /* В противном случае используем обычный printf */
    #define PRINTF_S(...) printf(__VA_ARGS__)
#endif
