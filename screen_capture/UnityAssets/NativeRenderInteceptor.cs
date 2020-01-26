using System;
using System.Collections;
using System.Runtime.InteropServices;
using UnityEngine;

/// <summary>
/// Copy RenderingInterceptor.dll from target/debug into Plugins/x86_64 and Attach this script to Main Camera
/// </summary>
public class NativeRenderInteceptor : MonoBehaviour
{
    [DllImport("RenderingInterceptor")]
    private static extern IntPtr rendering_event_ptr();
    [DllImport("RenderingInterceptor")]
    private static extern void set_render_buffer(IntPtr buf);

    // Start is called before the first frame update
    void Start()
    {
        this.StartCoroutine(this.RenderInterceptorAtFrameTail());
    }

    IEnumerator RenderInterceptorAtFrameTail()
    {
        while (true)
        {
            yield return new WaitForEndOfFrame();

            set_render_buffer(Graphics.activeColorBuffer.GetNativeRenderBufferPtr());
            GL.IssuePluginEvent(rendering_event_ptr(), 1);
        }
    }
}
