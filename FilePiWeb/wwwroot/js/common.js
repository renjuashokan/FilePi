window.commonHelper = {
    // Bootstrap Modal functions
    modal: {
        show: function (modalId, options = {}) {
            const modalElement = document.getElementById(modalId);
            if (modalElement) {
                const modal = new bootstrap.Modal(modalElement, options);
                modal.show();
            }
        },

        hide: function (modalId) {
            const modalElement = document.getElementById(modalId);
            if (modalElement) {
                const modal = bootstrap.Modal.getInstance(modalElement);
                if (modal) {
                    modal.hide();
                }
            }
        },

        toggle: function (modalId, options = {}) {
            const modalElement = document.getElementById(modalId);
            if (modalElement) {
                const modal = new bootstrap.Modal(modalElement, options);
                modal.toggle();
            }
        },

        dispose: function (modalId) {
            const modalElement = document.getElementById(modalId);
            if (modalElement) {
                const modal = bootstrap.Modal.getInstance(modalElement);
                if (modal) {
                    modal.dispose();
                }
            }
        }
    },

    // Navigation functions
    navigation: {
        openInNewTab: function (url) {
            window.open(url, '_blank', 'noopener,noreferrer');
        },

        openInNewWindow: function (url, windowName = '_blank', features = 'noopener,noreferrer') {
            window.open(url, windowName, features);
        },

        downloadFile: function (url) {
            window.open(url, '_blank');
        }
    },

    // Utility functions
    utilities: {
        focusElement: function (elementId) {
            const element = document.getElementById(elementId);
            if (element) {
                element.focus();
            }
        },

        scrollToElement: function (elementId) {
            const element = document.getElementById(elementId);
            if (element) {
                element.scrollIntoView({behavior: 'smooth'});
            }
        },

        copyToClipboard: function (text) {
            return navigator.clipboard.writeText(text).then(() => true).catch(() => false);
        }
    }
};
