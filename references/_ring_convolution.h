
/**
 * @brief Рассчитывает корректный индекс для кольцевых буферов.
 *
 * @param i индекс элемента.
 * @param h индекс головы буфера.
 * @param sz размер буфера.
 *
 */
#define _CALC_CBUF_I(i, h, sz)  (  ( (ptrdiff_t)(h) - (ptrdiff_t)(i) ) % (ptrdiff_t)(sz) + (sz)  ) % (sz)

/**
 * @brief Рассчитывает корректный индекс для хвостатых буферов, игнорируя хвост.
 *
 * @param i индекс элемента.
 * @param h индекс головы буфера.
 * @param tl длина хвоста.
 * @param sz размер буфера.
 *
 */
#define _CALC_CBUF_TAIL_I(i, h, tl, sz) (   (ptrdiff_t)(h) - (ptrdiff_t)(  (i)%( (ptrdiff_t)(sz) - (ptrdiff_t)(tl) )+( (ptrdiff_t)(sz) - (ptrdiff_t)(tl) )  )%(  (ptrdiff_t)(sz)-(ptrdiff_t)(tl)  ) + (sz)   )%(sz)

/**
 * @brief Рассчитывает корректный индекс элемента из стэка буферов.
 *
 * @param i индекс буфера.
 * @param j индекс элемента в буфере.
 * @param h индекс головы буфера.
 * @param sz размер буфера.
 *
 */
#define _CALC_CBUF_STACK_I(i, j, h, sz) (i)*(sz) + _CALC_CBUF_I(j, h, sz)

/**
 * @brief Рассчитывает корректный индекс элемента из стэка буферов.
 *
 * @param i индекс буфера.
 * @param j индекс элемента в буфере.
 * @param h индекс головы буфера.
 * @param tl длина хвоста.
 * @param sz размер буфера.
 *
 */
#define _CALC_CBUF_STACK_TAIL_I(i, j, h, tl, sz) (i)*(sz) + _CALC_CBUF_TAIL_I(j, h, tl, sz)
