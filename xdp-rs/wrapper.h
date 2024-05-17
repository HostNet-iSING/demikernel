// Only need the basic windows types and assertions.
#define WIN32_LEAN_AND_MEAN
#define NOGDICAPMASKS
#define NOVIRTUALKEYCODES
#define NOWINMESSAGES
#define NOWINSTYLES
#define NOSYSMETRICS
#define NOMENUS
#define NOICONS
#define NOKEYSTATES
#define NOSYSCOMMANDS
#define NORASTEROPS
#define NOSHOWWINDOW
#define OEMRESOURCE
#define NOATOM
#define NOCLIPBOARD
#define NOCOLOR
#define NOCTLMGR
#define NODRAWTEXT
#define NOGDI
#define NOKERNEL
#define NOUSER
#define NONLS
#define NOMB
#define NOMEMMGR
#define NOMETAFILE
#define NOMINMAX
#define NOMSG
#define NOOPENFILE
#define NOSCROLL
#define NOSERVICE
#define NOSOUND
#define NOTEXTMETRIC
#define NOWH
#define NOWINOFFSETS
#define NOCOMM
#define NOKANJI
#define NOHELP
#define NOPROFILER
#define NODEFERWINDOWPOS
#define NOMCX
#include <Windows.h>

#include <afxdp.h>
//#include <afxdp_helper.h>
#include <xdpapi.h>

// non-inlined load/unload functions.
HRESULT
_XdpLoadApi(
    _In_ UINT32 XdpApiVersion,
    _Out_ XDP_LOAD_API_CONTEXT *XdpLoadApiContext,
    _Out_ CONST XDP_API_TABLE **XdpApiTable
    );

VOID
_XdpUnloadApi(
    _In_ XDP_LOAD_API_CONTEXT XdpLoadApiContext,
    _In_ CONST XDP_API_TABLE *XdpApiTable
    );
