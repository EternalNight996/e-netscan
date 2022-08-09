#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <Windows.h>
#include <psapi.h>
#include <powrprof.h>
#include "info.h"
#define LEN 20
#define MAXPROCESSES 1024
static const char *os_type = "Windows";

/* 获取显示屏分辨率 */
MonitorInfo get_monitor_info(void)
{
    MonitorInfo mi;
    mi.xscreen = GetSystemMetrics(SM_CXSCREEN);
    mi.yscreen = GetSystemMetrics(SM_CYSCREEN);
    mi.cx_fullscreen = GetSystemMetrics(SM_CXFULLSCREEN);
    mi.cy_fullscreen = GetSystemMetrics(SM_CYFULLSCREEN);
    mi.cxvirtual_screen = GetSystemMetrics(SM_CXVIRTUALSCREEN);
    mi.cyvirtual_screen = GetSystemMetrics(SM_CYVIRTUALSCREEN);
    mi.xvirtual_screen = GetSystemMetrics(SM_XVIRTUALSCREEN);
    mi.yvirtual_screen = GetSystemMetrics(SM_YVIRTUALSCREEN);
    return mi;
}

/* 获取内存信息 */
MemInfo get_mem_info(void)
{
    MEMORYSTATUSEX stat;
    /* DWORDLONG size; */
    MemInfo mi;

    stat.dwLength = sizeof(stat);
    if (GlobalMemoryStatusEx(&stat))
    {
        mi.total = stat.ullTotalPhys / 1024;
        mi.avail = 0;
        mi.free = stat.ullAvailPhys / 1024;
        mi.cached = 0;
        mi.buffers = 0;
        mi.swap_total = (stat.ullTotalPageFile - stat.ullTotalPhys) / 1024;
        mi.swap_free = (stat.ullAvailPageFile - stat.ullAvailPhys) / 1024;
        if (mi.swap_free > mi.swap_total)
        {
            mi.swap_free = mi.swap_total;
        }
    }
    else
    {
        memset(&mi, 0, sizeof(mi));
    }
    return mi;
}
