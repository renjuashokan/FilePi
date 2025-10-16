using Microsoft.JSInterop;

namespace FilePiWeb.Extensions;

public static class JSRuntimeCommonExtensions
{
    /// <summary>
    ///     Shows a Bootstrap modal by its element ID
    /// </summary>
    /// <param name="jsRuntime">The JSRuntime instance</param>
    /// <param name="modalId">The ID of the modal element</param>
    /// <returns>A task representing the asynchronous operation</returns>
    public static async Task ShowBootstrapModalAsync(this IJSRuntime jsRuntime, string modalId)
    {
        await jsRuntime.InvokeVoidAsync("commonHelper.modal.show", modalId);
    }

    /// <summary>
    ///     Shows a Bootstrap modal with custom options
    /// </summary>
    /// <param name="jsRuntime">The JSRuntime instance</param>
    /// <param name="modalId">The ID of the modal element</param>
    /// <param name="backdrop">Whether to show backdrop ('static', true, or false)</param>
    /// <param name="keyboard">Whether to close modal on escape key</param>
    /// <param name="focus">Whether to focus modal when shown</param>
    /// <returns>A task representing the asynchronous operation</returns>
    public static async Task ShowBootstrapModalAsync(this IJSRuntime jsRuntime, string modalId,
        string backdrop = "true", bool keyboard = true, bool focus = true)
    {
        var options = new { backdrop, keyboard, focus };
        await jsRuntime.InvokeVoidAsync("commonHelper.modal.show", modalId, options);
    }

    /// <summary>
    ///     Hides a Bootstrap modal by its element ID
    /// </summary>
    /// <param name="jsRuntime">The JSRuntime instance</param>
    /// <param name="modalId">The ID of the modal element</param>
    /// <returns>A task representing the asynchronous operation</returns>
    public static async Task HideBootstrapModalAsync(this IJSRuntime jsRuntime, string modalId)
    {
        await jsRuntime.InvokeVoidAsync("commonHelper.modal.hide", modalId);
    }

    /// <summary>
    ///     Toggles a Bootstrap modal by its element ID
    /// </summary>
    /// <param name="jsRuntime">The JSRuntime instance</param>
    /// <param name="modalId">The ID of the modal element</param>
    /// <returns>A task representing the asynchronous operation</returns>
    public static async Task ToggleBootstrapModalAsync(this IJSRuntime jsRuntime, string modalId)
    {
        await jsRuntime.InvokeVoidAsync("commonHelper.modal.toggle", modalId);
    }

    /// <summary>
    ///     Disposes a Bootstrap modal by its element ID
    /// </summary>
    /// <param name="jsRuntime">The JSRuntime instance</param>
    /// <param name="modalId">The ID of the modal element</param>
    /// <returns>A task representing the asynchronous operation</returns>
    public static async Task DisposeBootstrapModalAsync(this IJSRuntime jsRuntime, string modalId)
    {
        await jsRuntime.InvokeVoidAsync("commonHelper.modal.dispose", modalId);
    }

    /// <summary>
    ///     Opens a URL in a new tab
    /// </summary>
    /// <param name="jsRuntime">The JSRuntime instance</param>
    /// <param name="url">The URL to open</param>
    /// <returns>A task representing the asynchronous operation</returns>
    public static async Task OpenInNewTabAsync(this IJSRuntime jsRuntime, string url)
    {
        await jsRuntime.InvokeVoidAsync("commonHelper.navigation.openInNewTab", url);
    }

    /// <summary>
    ///     Opens a URL in a new window with custom options
    /// </summary>
    /// <param name="jsRuntime">The JSRuntime instance</param>
    /// <param name="url">The URL to open</param>
    /// <param name="windowName">The name of the window</param>
    /// <param name="features">Window features</param>
    /// <returns>A task representing the asynchronous operation</returns>
    public static async Task OpenInNewWindowAsync(this IJSRuntime jsRuntime, string url,
        string windowName = "_blank", string features = "noopener,noreferrer")
    {
        await jsRuntime.InvokeVoidAsync("commonHelper.navigation.openInNewWindow", url, windowName, features);
    }

    /// <summary>
    ///     Downloads a file by opening the URL
    /// </summary>
    /// <param name="jsRuntime">The JSRuntime instance</param>
    /// <param name="url">The URL of the file to download</param>
    /// <returns>A task representing the asynchronous operation</returns>
    public static async Task DownloadFileAsync(this IJSRuntime jsRuntime, string url)
    {
        await jsRuntime.InvokeVoidAsync("commonHelper.navigation.downloadFile", url);
    }

    /// <summary>
    ///     Focuses an element by its ID
    /// </summary>
    /// <param name="jsRuntime">The JSRuntime instance</param>
    /// <param name="elementId">The ID of the element to focus</param>
    /// <returns>A task representing the asynchronous operation</returns>
    public static async Task FocusElementAsync(this IJSRuntime jsRuntime, string elementId)
    {
        await jsRuntime.InvokeVoidAsync("commonHelper.utilities.focusElement", elementId);
    }

    /// <summary>
    ///     Scrolls to an element by its ID
    /// </summary>
    /// <param name="jsRuntime">The JSRuntime instance</param>
    /// <param name="elementId">The ID of the element to scroll to</param>
    /// <returns>A task representing the asynchronous operation</returns>
    public static async Task ScrollToElementAsync(this IJSRuntime jsRuntime, string elementId)
    {
        await jsRuntime.InvokeVoidAsync("commonHelper.utilities.scrollToElement", elementId);
    }

    /// <summary>
    ///     Copies text to the clipboard
    /// </summary>
    /// <param name="jsRuntime">The JSRuntime instance</param>
    /// <param name="text">The text to copy</param>
    /// <returns>A task representing the asynchronous operation, returns true if successful</returns>
    public static async Task<bool> CopyToClipboardAsync(this IJSRuntime jsRuntime, string text)
    {
        return await jsRuntime.InvokeAsync<bool>("commonHelper.utilities.copyToClipboard", text);
    }
}
