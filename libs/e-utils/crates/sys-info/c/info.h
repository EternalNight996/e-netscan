#ifndef INFO_H_
#define INFO_H_

/* 内存信息模块 */
typedef struct MemInfo
{
    unsigned long long total;
    unsigned long long free;
    unsigned long long avail;

    unsigned long long buffers;
    unsigned long long cached;

    unsigned long long swap_total;
    unsigned long long swap_free;
} MemInfo;
MemInfo get_mem_info(void);

/* 显示器信息模块 */
typedef struct MonitorInfo
{
    int xscreen;
    int yscreen;
    int cy_fullscreen;
    int cx_fullscreen;
    int cxvirtual_screen;
    int cyvirtual_screen;
    int xvirtual_screen;
    int yvirtual_screen;
} MonitorInfo;
MonitorInfo get_monitor_info(void);

#endif