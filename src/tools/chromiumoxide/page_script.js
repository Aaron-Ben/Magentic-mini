var WebSurfer = WebSurfer || (function () {
    /**
     * WebSurfer - A JavaScript module for analyzing web page content and interactive elements
     *
     * This module provides functionality to:
     * - Detect and analyze interactive elements (buttons, links, inputs etc.)
     * - Track viewport information and scrolling
     * - Extract page metadata (JSON-LD, microdata, meta tags)
     * - Get visible text content
     *
     * The module handles both regular DOM elements and shadow DOM components.
     */

    let nextLabel = 10;

    let roleMapping = {
        "a": "link",
        "area": "link",
        "button": "button",
        "input, type=button": "button",
        "input, type=checkbox": "checkbox",
        "input, type=email": "textbox",
        "input, type=number": "spinbutton",
        "input, type=radio": "radio",
        "input, type=range": "slider",
        "input, type=reset": "button",
        "input, type=search": "searchbox",
        "input, type=submit": "button",
        "input, type=tel": "textbox",
        "input, type=text": "textbox",
        "input, type=url": "textbox",
        "search": "search",
        "select": "combobox",
        "option": "option",
        "textarea": "textbox"
    };

    let getCursor = function (elm) {
        return window.getComputedStyle(elm)["cursor"];
    };

    let isVisible = function (element) {
        return !!(element.offsetWidth || element.offsetHeight || element.getClientRects().length);
    };

    /**
     * Finds interactive elements in the regular DOM (excluding Shadow DOM)
     * Looks for elements that are:
     * 1. Standard interactive elements (inputs, buttons, links)
     * 2. Elements with ARIA roles indicating interactivity
     * 3. Elements with cursor styles suggesting interactivity
     *
     * @returns {Array} Array of DOM elements that are deemed interactive
     */
    let getInteractiveElementsNoShaddow = function () {
        let results = []
        let roles = ["scrollbar", "searchbox", "slider", "spinbutton", "switch", "tab", "treeitem", "button", "checkbox", "gridcell", "link", "menuitem", "menuitemcheckbox", "menuitemradio", "option", "progressbar", "radio", "textbox", "combobox", "menu", "tree", "treegrid", "grid", "listbox", "radiogroup", "widget"];
        let inertCursors = ["auto", "default", "none", "text", "vertical-text", "not-allowed", "no-drop"];

        // Get the main interactive elements
        let nodeList = document.querySelectorAll("input, select, textarea, button, [href], [onclick], [contenteditable], [tabindex]:not([tabindex='-1'])");
        for (let i = 0; i < nodeList.length; i++) { // Copy to something mutable
            // make sure not disabled and visible
            if (nodeList[i].disabled || !isVisible(nodeList[i])) {
                continue;
            }
            results.push(nodeList[i]);
        }

        // Anything not already included that has a suitable role
        nodeList = document.querySelectorAll("[role]");
        for (let i = 0; i < nodeList.length; i++) { // Copy to something mutable
            // make sure not disabled and visible
            if (nodeList[i].disabled || !isVisible(nodeList[i])) {
                continue;
            }
            if (results.indexOf(nodeList[i]) == -1) {
                let role = nodeList[i].getAttribute("role");
                if (roles.indexOf(role) > -1) {
                    results.push(nodeList[i]);
                }
            }
        }

        // Any element that changes the cursor to something implying interactivity
        nodeList = document.querySelectorAll("*");
        for (let i = 0; i < nodeList.length; i++) {
            let node = nodeList[i];
            if (node.disabled || !isVisible(node)) {
                continue;
            }

            // Cursor is default, or does not suggest interactivity
            let cursor = getCursor(node);
            if (inertCursors.indexOf(cursor) >= 0) {
                continue;
            }

            // Move up to the first instance of this cursor change
            let parent = node.parentNode;
            while (parent && getCursor(parent) == cursor) {
                node = parent;
                parent = node.parentNode;
            }

            // Add the node if it is new
            if (results.indexOf(node) == -1) {
                results.push(node);
            }
        }

        return results;
    };

    /**
     * Recursively gathers elements matching specified roles from both regular DOM and Shadow DOM
     * @param {Array} roles - Array of role selectors to match
     * @param {Document|ShadowRoot} root - Root element to start search from
     * @returns {Array} Array of matching elements
     */
    function gatherAllElements(roles, root = document) {
        const elements = [];
        const stack = [root];
        const selector = roles.join(",");

        while (stack.length > 0) {
            const currentRoot = stack.pop();

            // Add elements at current level
            elements.push(...Array.from(currentRoot.querySelectorAll(selector)));

            // Add shadow roots to stack
            currentRoot.querySelectorAll("*").forEach(el => {
                if (el.shadowRoot && el.shadowRoot.mode === "open") {
                    stack.push(el.shadowRoot);
                }
            });
        }

        return elements;
    }

    /**
     * Gets all interactive elements from both regular DOM and Shadow DOM
     * Filters elements to ensure they are visible and accessible
     * @returns {Array} Array of interactive elements
     */
    let getInteractiveElements = function () {
        // Get all elements that are interactive without the shadow DOM
        const interactive_roles = ["input", "option", "select", "textarea", "button", "href", "onclick", "contenteditable", "tabindex:not([tabindex='-1'])"];

        let results = [];

        let elements_no_shaddow = getInteractiveElementsNoShaddow();
        for (let i = 0; i < elements_no_shaddow.length; i++) {
            if (results.indexOf(elements_no_shaddow[i]) == -1) {
                // check if it has a rect
                let rects = elements_no_shaddow[i].getClientRects();
                for (const rect of rects) {
                    let x = rect.left + rect.width / 2;
                    let y = rect.top + rect.height / 2;
                    if (isTopmost(elements_no_shaddow[i], x, y)) {
                        results.push(elements_no_shaddow[i]);
                        break;
                    }
                }
            }
        }

        // From the shadow DOM get all interactive elements and options that are not in the no shadow list
        let elements_all = gatherAllElements(interactive_roles);

        // Filter and process interactive elements
        elements_all.forEach(element => {
            // if file, auto add
            if (element.tagName.toLowerCase() === "input" && element.getAttribute("type") == "file") {
                results.push(element);
                return;
            }
            // if option  auto add
            if (element.tagName.toLowerCase() === "option") {
                results.push(element);
                return;
            }
            if (element.disabled || !isVisible(element)) {
                return;
            }

            if (interactive_roles.includes(element.tagName.toLowerCase())) {
                results.push(element);
            }
        });

        return results;
    };

    /**
     * Assigns unique identifiers to interactive elements
     * @param {Array} elements - Array of elements to label
     * @returns {Array} Updated array of interactive elements
     */
    let labelElements = function (elements) {
        for (let i = 0; i < elements.length; i++) {
            if (!elements[i].hasAttribute("__elementId")) {
                elements[i].setAttribute("__elementId", "" + (nextLabel++));
            }
        }
        return getInteractiveElements();
    };

    /**
     * Checks if an element is the topmost element at given coordinates
     * @param {Element} element - Element to check
     * @param {number} x - X coordinate
     * @param {number} y - Y coordinate
     * @returns {boolean} True if element is topmost at coordinates
     */
    let isTopmost = function (element, x, y) {
        let hit = document.elementFromPoint(x, y);

        // Hack to handle elements outside the viewport
        if (hit === null) {
            return true;
        }

        while (hit) {
            if (hit == element) return true;
            hit = hit.parentNode;
        }
        return false;
    };

    let getFocusedElementId = function () {
        let elm = document.activeElement;
        while (elm) {
            if (elm.hasAttribute && elm.hasAttribute("__elementId")) {
                return elm.getAttribute("__elementId");
            }
            elm = elm.parentNode;
        }
        return null;
    };

    let trimmedInnerText = function (element) {
        if (!element) {
            return "";
        }
        let text = element.innerText;
        if (!text) {
            return "";
        }
        return text.trim();
    };

    let getApproximateAriaName = function (element) {
        if (element.hasAttribute("aria-label")) {
            return element.getAttribute("aria-label");
        }

        // check if element has span that is called label and grab the inner text
        if (element.querySelector("span.label")) {
            return element.querySelector("span.label").innerText;
        }

        // Check for aria labels
        if (element.hasAttribute("aria-labelledby")) {
            let buffer = "";
            let ids = element.getAttribute("aria-labelledby").split(" ");
            for (let i = 0; i < ids.length; i++) {
                let label = document.getElementById(ids[i]);
                if (label) {
                    buffer = buffer + " " + trimmedInnerText(label);
                }
            }
            return buffer.trim();
        }

        if (element.hasAttribute("aria-label")) {
            return element.getAttribute("aria-label");
        }

        // Check for labels
        if (element.hasAttribute("id")) {
            let label_id = element.getAttribute("id");
            let label = "";
            try {
                // Escape special characters in the ID
                let escaped_id = CSS.escape(label_id);
                let labels = document.querySelectorAll(`label[for="${escaped_id}"]`);
                for (let j = 0; j < labels.length; j++) {
                    label += labels[j].innerText + " ";
                }
                label = label.trim();
                if (label != "") {
                    return label;
                }
            } catch (e) {
                console.warn("Error finding label for element:", e);
            }
        }

        if (element.hasAttribute("name")) {
            return element.getAttribute("name");
        }

        if (element.parentElement && element.parentElement.tagName == "LABEL") {
            return element.parentElement.innerText;
        }

        // Check for alt text or titles
        if (element.hasAttribute("alt")) {
            return element.getAttribute("alt")
        }

        if (element.hasAttribute("title")) {
            return element.getAttribute("title")
        }

        return trimmedInnerText(element);
    };

    let getApproximateAriaRole = function (element) {
        let tag = element.tagName.toLowerCase();
        if (tag == "input" && element.hasAttribute("type")) {
            tag = tag + ", type=" + element.getAttribute("type");
        }

        if (element.hasAttribute("role")) {
            return [element.getAttribute("role"), tag];
        }
        else if (tag in roleMapping) {
            return [roleMapping[tag], tag];
        }
        else {
            return ["", tag];
        }
    };

    /**
     * Gets information about all interactive elements including their:
     * - Position and dimensions
     * - ARIA roles and names
     * - Tag names
     * - Scrollability
     *
     * @returns {Object} Map of element IDs to their properties
     */
    let getInteractiveRects = function () {
        let elements = labelElements(getInteractiveElements());
        let results = {};
        for (let i = 0; i < elements.length; i++) {
            let key = elements[i].getAttribute("__elementId");
            let rects = elements[i].getBoundingClientRect();

            // Skip options unless their select is focused
            if (elements[i].tagName.toLowerCase() === "option") {

                let select_focused = false;
                let select = elements[i].closest("select");
                if (select && select.hasAttribute("__elementId") &&
                    getFocusedElementId() === select.getAttribute("__elementId")) {
                    select_focused = true;
                }
                // check if option is visible without select being focused
                let option_visible = false;
                if (isVisible(elements[i])) {
                    option_visible = true;
                }
                // check if select is expanded even if not focused
                let select_expanded = false;
                if (select && select.hasAttribute("open")) {
                    select_expanded = true;
                }
                if (!(select_focused || option_visible || select_expanded)) {
                    continue;
                }
            }

            let ariaRole = getApproximateAriaRole(elements[i]);
            let ariaName = getApproximateAriaName(elements[i]);
            let vScrollable = elements[i].scrollHeight - elements[i].clientHeight >= 1;

            let record = {
                "tag_name": ariaRole[1],
                "role": ariaRole[0],
                "aria-name": ariaName,
                "v-scrollable": vScrollable,
                "rects": []
            };

            if (rects.length > 0) {
                for (const rect of rects) {
                    let x = rect.left + rect.width / 2;
                    let y = rect.top + rect.height / 2;
                    if (isTopmost(elements[i], x, y)) {
                        record["rects"].push(JSON.parse(JSON.stringify(rect)));
                    }
                }
            }
            else {
                record["rects"].push(JSON.parse(JSON.stringify(rects)));
            }

            results[key] = record;
        }
        return results;
    };

    /**
     * Gets current viewport information including dimensions and scroll positions
     * @returns {Object} Viewport properties
     */
    let getVisualViewport = function () {
        let vv = window.visualViewport;
        let de = document.documentElement;
        return {
            "height": vv ? vv.height : 0,
            "width": vv ? vv.width : 0,
            "offsetLeft": vv ? vv.offsetLeft : 0,
            "offsetTop": vv ? vv.offsetTop : 0,
            "pageLeft": vv ? vv.pageLeft : 0,
            "pageTop": vv ? vv.pageTop : 0,
            "scale": vv ? vv.scale : 0,
            "clientWidth": de ? de.clientWidth : 0,
            "clientHeight": de ? de.clientHeight : 0,
            "scrollWidth": de ? de.scrollWidth : 0,
            "scrollHeight": de ? de.scrollHeight : 0
        };
    };

    let _getMetaTags = function () {
        let meta = document.querySelectorAll("meta");
        let results = {};
        for (let i = 0; i < meta.length; i++) {
            let key = null;
            if (meta[i].hasAttribute("name")) {
                key = meta[i].getAttribute("name");
            }
            else if (meta[i].hasAttribute("property")) {
                key = meta[i].getAttribute("property");
            }
            else {
                continue;
            }
            if (meta[i].hasAttribute("content")) {
                results[key] = meta[i].getAttribute("content");
            }
        }
        return results;
    };

    let _getJsonLd = function () {
        let jsonld = [];
        let scripts = document.querySelectorAll('script[type="application/ld+json"]');
        for (let i = 0; i < scripts.length; i++) {
            jsonld.push(scripts[i].innerHTML.trim());
        }
        return jsonld;
    };

    // From: https://www.stevefenton.co.uk/blog/2022/12/parse-microdata-with-javascript/
    let _getMicrodata = function () {
        function sanitize(input) {
            return input.replace(/\s/gi, ' ').trim();
        }

        function addValue(information, name, value) {
            if (information[name]) {
                if (typeof information[name] === 'array') {
                    information[name].push(value);
                } else {
                    const arr = [];
                    arr.push(information[name]);
                    arr.push(value);
                    information[name] = arr;
                }
            } else {
                information[name] = value;
            }
        }

        function traverseItem(item, information) {
            const children = item.children;

            for (let i = 0; i < children.length; i++) {
                const child = children[i];

                if (child.hasAttribute('itemscope')) {
                    if (child.hasAttribute('itemprop')) {
                        const itemProp = child.getAttribute('itemprop');
                        const itemType = child.getAttribute('itemtype');

                        const childInfo = {
                            itemType: itemType
                        };

                        traverseItem(child, childInfo);

                        itemProp.split(' ').forEach(propName => {
                            addValue(information, propName, childInfo);
                        });
                    }

                } else if (child.hasAttribute('itemprop')) {
                    const itemProp = child.getAttribute('itemprop');
                    itemProp.split(' ').forEach(propName => {
                        if (propName === 'url') {
                            addValue(information, propName, child.href);
                        } else {
                            addValue(information, propName, sanitize(child.getAttribute("content") || child.content || child.textContent || child.src || ""));
                        }
                    });
                    traverseItem(child, information);
                } else {
                    traverseItem(child, information);
                }
            }
        }

        const microdata = [];

        document.querySelectorAll("[itemscope]").forEach(function (elem, i) {
            const itemType = elem.getAttribute('itemtype');
            const information = {
                itemType: itemType
            };
            traverseItem(elem, information);
            microdata.push(information);
        });

        return microdata;
    };

    let getPageMetadata = function () {
        let jsonld = _getJsonLd();
        let metaTags = _getMetaTags();
        let microdata = _getMicrodata();
        let results = {}
        if (jsonld.length > 0) {
            try {
                results["jsonld"] = JSON.parse(jsonld);
            }
            catch (e) {
                results["jsonld"] = jsonld;
            }
        }
        if (microdata.length > 0) {
            results["microdata"] = microdata;
        }
        for (let key in metaTags) {
            if (metaTags.hasOwnProperty(key)) {
                results["meta_tags"] = metaTags;
                break;
            }
        }
        return results;
    };

    /**
     * Extracts all visible text content from the viewport
     * Preserves basic formatting with newlines for block elements
     * @returns {string} Visible text content
     */
    let getVisibleText = function () {
        // Get the window's current viewport boundaries
        const viewportHeight = window.innerHeight || document.documentElement.clientHeight;
        const viewportWidth = window.innerWidth || document.documentElement.clientWidth;

        let textInView = "";
        const walker = document.createTreeWalker(
            document.body,
            NodeFilter.SHOW_TEXT,
            null,
            false
        );

        while (walker.nextNode()) {
            const textNode = walker.currentNode;
            // Create a range to retrieve bounding rectangles of the current text node
            const range = document.createRange();
            range.selectNodeContents(textNode);

            const rects = range.getClientRects();

            // Check if any rect is inside (or partially inside) the viewport
            for (const rect of rects) {
                const isVisible =
                    rect.width > 0 &&
                    rect.height > 0 &&
                    rect.bottom >= 0 &&
                    rect.right >= 0 &&
                    rect.top <= viewportHeight &&
                    rect.left <= viewportWidth;

                if (isVisible) {
                    textInView += textNode.nodeValue.replace(/\s+/g, " ");
                    // Is the parent a block element?
                    if (textNode.parentNode) {
                        const parent = textNode.parentNode;
                        const style = window.getComputedStyle(parent);
                        if (["inline", "hidden", "none"].indexOf(style.display) === -1) {
                            textInView += "\n";
                        }
                    }
                    break; // No need to check other rects once found visible
                }
            }
        }

        // Remove blank lines from textInView
        textInView = textInView.replace(/^\s*\n/gm, "").trim().replace(/\n+/g, "\n");
        return textInView;
    };

    /**
     * Converts HTML content to simplified Markdown format
     * @returns {string} Markdown representation of the page
     */
    let getPageMarkdown = function() {
        try {
            // 简化的 HTML 到 Markdown 转换
            let content = document.body.cloneNode(true);
            
            // 移除脚本和样式标签
            content.querySelectorAll('script, style, nav, footer, aside').forEach(el => el.remove());
            
            let markdown = '';
            
            function processElement(element) {
                if (!element || !element.tagName) {
                    return '';
                }
                
                const tagName = element.tagName.toLowerCase();
                const text = element.textContent ? element.textContent.trim() : '';
                
                switch (tagName) {
                    case 'h1':
                        return text ? `# ${text}\n\n` : '';
                    case 'h2':
                        return text ? `## ${text}\n\n` : '';
                    case 'h3':
                        return text ? `### ${text}\n\n` : '';
                    case 'h4':
                        return text ? `#### ${text}\n\n` : '';
                    case 'h5':
                        return text ? `##### ${text}\n\n` : '';
                    case 'h6':
                        return text ? `###### ${text}\n\n` : '';
                    case 'p':
                        return text ? `${text}\n\n` : '';
                    case 'a':
                        const href = element.getAttribute('href');
                        return href && text ? `[${text}](${href})` : text;
                    case 'strong':
                    case 'b':
                        return text ? `**${text}**` : '';
                    case 'em':
                    case 'i':
                        return text ? `*${text}*` : '';
                    case 'code':
                        return text ? `\`${text}\`` : '';
                    case 'pre':
                        return text ? `\`\`\`\n${text}\n\`\`\`\n\n` : '';
                    case 'ul':
                    case 'ol':
                        let listItems = '';
                        const listElements = element.querySelectorAll('li');
                        for (let i = 0; i < listElements.length; i++) {
                            const li = listElements[i];
                            const prefix = tagName === 'ol' ? `${i + 1}. ` : '- ';
                            const liText = li.textContent ? li.textContent.trim() : '';
                            if (liText) {
                                listItems += `${prefix}${liText}\n`;
                            }
                        }
                        return listItems ? `${listItems}\n` : '';
                    case 'blockquote':
                        return text ? `> ${text}\n\n` : '';
                    case 'br':
                        return '\n';
                    case 'hr':
                        return '---\n\n';
                    case 'div':
                    case 'section':
                    case 'article':
                        // 对于容器元素，不直接处理，让递归处理子元素
                        return null;
                    default:
                        return null; // 让递归处理
                }
            }
            
            function walkNodes(node) {
                if (!node) return;
                
                if (node.nodeType === Node.TEXT_NODE) {
                    const text = node.textContent ? node.textContent.trim() : '';
                    if (text && text.length > 0) {
                        // 避免添加重复的空格
                        if (markdown && !markdown.endsWith(' ') && !markdown.endsWith('\n')) {
                            markdown += ' ';
                        }
                        markdown += text;
                    }
                } else if (node.nodeType === Node.ELEMENT_NODE) {
                    const processed = processElement(node);
                    
                    if (processed !== null && processed !== undefined) {
                        // 直接使用处理后的结果
                        markdown += processed;
                    } else {
                        // 递归处理子节点
                        const children = node.childNodes;
                        for (let i = 0; i < children.length; i++) {
                            walkNodes(children[i]);
                        }
                        
                        // 检查是否为块级元素，需要添加换行
                        const tagName = node.tagName ? node.tagName.toLowerCase() : '';
                        const blockElements = ['div', 'p', 'section', 'article', 'header', 'main'];
                        if (blockElements.includes(tagName) && markdown && !markdown.endsWith('\n')) {
                            markdown += '\n';
                        }
                    }
                }
            }
            
            walkNodes(content);
            
            // 清理多余的空行和空格
            markdown = markdown
                .replace(/[ \t]+/g, ' ')  // 多个空格替换为单个空格
                .replace(/\n{3,}/g, '\n\n')  // 多个换行替换为双换行
                .replace(/^\s+|\s+$/g, '');  // 去除首尾空白
            
            return markdown || 'No content found';
            
        } catch (error) {
            console.error('Error in getPageMarkdown:', error);
            return 'Error extracting page content: ' + error.message;
        }
    };

    /**
     * Gets interactive elements (compatibility wrapper for getInteractiveRects)
     * @returns {Array} Array of interactive elements information
     */
    let getInteractiveElementsArray = function() {
        try {
            const rects = getInteractiveRects();
            const result = [];
            
            for (const id in rects) {
                if (rects.hasOwnProperty(id)) {
                    result.push({
                        id: id,
                        tag_name: rects[id].tag_name,
                        role: rects[id].role,
                        'aria-name': rects[id]['aria-name'],
                        'v-scrollable': rects[id]['v-scrollable'],
                        rects: rects[id].rects
                    });
                }
            }
            
            return result;
        } catch (error) {
            console.error('Error in getInteractiveElementsArray:', error);
            return [];
        }
    };

    // Public API
    return {
        getInteractiveRects: getInteractiveRects,
        getInteractiveElements: getInteractiveElementsArray,
        getVisualViewport: getVisualViewport,
        getFocusedElementId: getFocusedElementId,
        getPageMetadata: getPageMetadata,
        getVisibleText: getVisibleText,
        getPageMarkdown: getPageMarkdown,
    };
})();
