(() => {
	const state = {
		activeTab: "direct",
		rawHtml: "",
		selectedFile: null,
		messages: [],
		showSource: true,
		warningsOnly: true,
		loading: false,
	};

	const _sampleDoc = `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>Sample Valid HTML5</title>
</head>
<body>
  <main>
    <h1>Welcome to vnu-rs</h1>
    <p>This document is clean and valid according to the W3C HTML5 specification.</p>
  </main>
</body>
</html>`;

	const sampleInvalidDoc = `<!DOCTYPE html>
<html>
<head>
  <title>Invalid Document Test</title>
</head>
<body>
  <a href="https://example.com" target="_blank">Missing rel=noopener</a>
  <img src="logo.png">
  <div id="duplicate">First</div>
  <div id="duplicate">Second</div>
  <table>
    <tr><td>Data without table structure properly checked</td></tr>
  </table>
</body>
</html>`;

	// Elements
	const tabBtns = document.querySelectorAll(".tab-btn");
	const tabPanes = document.querySelectorAll(".tab-pane");
	const directEditor = document.getElementById("direct-editor");
	const fileDropzone = document.getElementById("file-dropzone");
	const fileInput = document.getElementById("file-input");
	const fileNameDisplay = document.getElementById("file-name");
	const urlInput = document.getElementById("url-input");
	const optShowSource = document.getElementById("opt-show-source");
	const optWarningsOnly = document.getElementById("opt-warnings-only");
	const btnValidate = document.getElementById("btn-validate");
	const btnSpinner = document.getElementById("btn-spinner");
	const btnText = document.getElementById("btn-text");
	const btnSample = document.getElementById("btn-sample");
	const btnClear = document.getElementById("btn-clear");
	const resultsArea = document.getElementById("results-area");
	const statusBanner = document.getElementById("status-banner");
	const statusTitle = document.getElementById("status-title");
	const statusMeta = document.getElementById("status-meta");
	const messagesList = document.getElementById("messages-list");
	const sourceWrap = document.getElementById("source-wrap");
	const sourceContainer = document.getElementById("source-container");

	// Tab switching
	tabBtns.forEach((btn) => {
		btn.addEventListener("click", () => {
			tabBtns.forEach((b) => {
				b.classList.remove("active");
			});
			tabPanes.forEach((p) => {
				p.classList.remove("active");
			});
			btn.classList.add("active");
			const tabId = btn.getAttribute("data-tab");
			state.activeTab = tabId;
			document.getElementById(`tab-${tabId}`).classList.add("active");
		});
	});

	// Options
	optShowSource.addEventListener("change", (e) => {
		state.showSource = e.target.checked;
		if (state.messages && state.rawHtml) renderSource();
	});
	optWarningsOnly.addEventListener("change", (e) => {
		state.warningsOnly = e.target.checked;
		if (state.messages) renderMessages();
	});

	// Sample and Clear
	btnSample.addEventListener("click", () => {
		state.activeTab = "direct";
		tabBtns[0].click();
		directEditor.value = sampleInvalidDoc;
	});
	btnClear.addEventListener("click", () => {
		directEditor.value = "";
		urlInput.value = "";
		state.selectedFile = null;
		fileInput.value = "";
		fileNameDisplay.classList.add("hidden");
		resultsArea.classList.add("hidden");
	});

	// File Dropzone
	fileDropzone.addEventListener("click", () => fileInput.click());
	fileInput.addEventListener("change", (e) => {
		if (e.target.files?.[0]) {
			handleFile(e.target.files[0]);
		}
	});
	fileDropzone.addEventListener("dragover", (e) => {
		e.preventDefault();
		fileDropzone.classList.add("dragover");
	});
	fileDropzone.addEventListener("dragleave", () =>
		fileDropzone.classList.remove("dragover"),
	);
	fileDropzone.addEventListener("drop", (e) => {
		e.preventDefault();
		fileDropzone.classList.remove("dragover");
		if (e.dataTransfer.files?.[0]) {
			handleFile(e.dataTransfer.files[0]);
		}
	});

	function handleFile(file) {
		state.selectedFile = file;
		fileNameDisplay.textContent = `Selected: ${file.name} (${(file.size / 1024).toFixed(1)} KB)`;
		fileNameDisplay.classList.remove("hidden");
	}

	// Validation
	btnValidate.addEventListener("click", async () => {
		let content = "";
		if (state.activeTab === "direct") {
			content = directEditor.value.trim();
			if (!content) {
				alert("Please paste or write HTML markup first.");
				return;
			}
		} else if (state.activeTab === "file") {
			if (!state.selectedFile) {
				alert("Please choose an HTML file first.");
				return;
			}
			try {
				content = await state.selectedFile.text();
			} catch (err) {
				alert(`Failed to read file: ${err.message}`);
				return;
			}
		} else if (state.activeTab === "url") {
			const url = urlInput.value.trim();
			if (!url) {
				alert("Please enter a valid URL.");
				return;
			}
			setLoading(true);
			try {
				const resp = await fetch(url);
				if (!resp.ok) throw new Error(`HTTP ${resp.status} ${resp.statusText}`);
				content = await resp.text();
			} catch (err) {
				setLoading(false);
				alert(
					"Could not fetch URL: " +
						err.message +
						"\nEnsure the target server permits cross-origin requests (CORS).",
				);
				return;
			}
		}

		state.rawHtml = content;
		await executeValidation(content);
	});

	function setLoading(val) {
		state.loading = val;
		if (val) {
			btnSpinner.classList.remove("hidden");
			btnText.textContent = "Validating...";
			btnValidate.disabled = true;
		} else {
			btnSpinner.classList.add("hidden");
			btnText.textContent = "Check HTML";
			btnValidate.disabled = false;
		}
	}

	async function executeValidation(html) {
		setLoading(true);
		const t0 = performance.now();
		try {
			const res = await fetch("/?out=json", {
				method: "POST",
				headers: { "Content-Type": "text/html; charset=utf-8" },
				body: html,
			});
			const t1 = performance.now();
			const elapsed = (t1 - t0).toFixed(1);

			if (!res.ok) {
				throw new Error(`Server returned ${res.status}`);
			}
			const data = await res.json();
			state.messages = data.messages || [];
			renderResults(elapsed);
		} catch (err) {
			alert(`Validation request failed: ${err.message}`);
		} finally {
			setLoading(false);
		}
	}

	function renderResults(elapsedMs) {
		resultsArea.classList.remove("hidden");
		const msgs = state.messages;
		const errors = msgs.filter((m) => m.type === "error");
		const warnings = msgs.filter(
			(m) => m.type === "info" && m.subType === "warning",
		);

		if (errors.length === 0 && warnings.length === 0) {
			statusBanner.className = "status-banner success";
			statusTitle.textContent =
				"Document checking completed. No errors or warnings to show.";
		} else {
			statusBanner.className = "status-banner error";
			statusTitle.textContent = `Found ${errors.length} error${errors.length === 1 ? "" : "s"}${warnings.length > 0 ? `, ${warnings.length} warning${warnings.length === 1 ? "" : "s"}` : ""}.`;
		}
		statusMeta.textContent = `⚡ ${elapsedMs} ms (Rust Engine)`;

		renderMessages();
		renderSource();
		resultsArea.scrollIntoView({ behavior: "smooth" });
	}

	function renderMessages() {
		messagesList.innerHTML = "";
		const filtered = state.messages.filter((m) => {
			if (!state.warningsOnly) return true;
			return (
				m.type === "error" || (m.type === "info" && m.subType === "warning")
			);
		});

		if (filtered.length === 0 && state.messages.length > 0) {
			const emptyNote = document.createElement("div");
			emptyNote.style.color = "var(--text-muted)";
			emptyNote.style.padding = "12px 0";
			emptyNote.textContent =
				"No errors or warnings. (Some informational notes were hidden by filter)";
			messagesList.appendChild(emptyNote);
			return;
		}

		filtered.forEach((m, _idx) => {
			const isWarn = m.type === "info" && m.subType === "warning";
			const isInfo = m.type === "info" && !m.subType;
			const kind = isWarn ? "warning" : isInfo ? "info" : "error";

			const card = document.createElement("div");
			card.className = `msg-card ${kind}`;

			const header = document.createElement("div");
			header.className = "msg-header";

			const badge = document.createElement("span");
			badge.className = "msg-badge";
			badge.textContent = kind;
			header.appendChild(badge);

			const loc = document.createElement("span");
			loc.className = "msg-location";
			const lineStr = m.lastLine ? `Line ${m.lastLine}` : "";
			const colStr = m.lastColumn ? `, Col ${m.lastColumn}` : "";
			loc.textContent = `${lineStr}${colStr}`;
			loc.title = "Click to jump to line in source";
			loc.addEventListener("click", () => scrollToLine(m.lastLine));
			header.appendChild(loc);

			card.appendChild(header);

			const text = document.createElement("div");
			text.className = "msg-text";
			text.textContent = m.message;
			card.appendChild(text);

			if (m.extract) {
				const extract = document.createElement("div");
				extract.className = "msg-extract";
				if (m.hiliteStart !== undefined && m.hiliteLength !== undefined) {
					const before = escapeHtml(m.extract.slice(0, m.hiliteStart));
					const hilite = escapeHtml(
						m.extract.slice(m.hiliteStart, m.hiliteStart + m.hiliteLength),
					);
					const after = escapeHtml(
						m.extract.slice(m.hiliteStart + m.hiliteLength),
					);
					extract.innerHTML = `${before}<mark>${hilite}</mark>${after}`;
				} else {
					extract.textContent = m.extract;
				}
				card.appendChild(extract);
			}

			messagesList.appendChild(card);
		});
	}

	function renderSource() {
		if (!state.showSource || !state.rawHtml) {
			sourceWrap.classList.add("hidden");
			return;
		}
		sourceWrap.classList.remove("hidden");
		sourceContainer.innerHTML = "";

		const lines = state.rawHtml.split("\n");
		const errorLines = new Set();
		state.messages.forEach((m) => {
			if (m.lastLine) errorLines.add(m.lastLine);
		});

		const frag = document.createDocumentFragment();
		lines.forEach((lineText, idx) => {
			const lineNum = idx + 1;
			const lineEl = document.createElement("div");
			lineEl.className = "source-line";
			lineEl.id = `source-line-${lineNum}`;
			if (errorLines.has(lineNum)) {
				lineEl.classList.add("has-error");
			}

			const numEl = document.createElement("div");
			numEl.className = "source-line-num";
			numEl.textContent = lineNum;

			const codeEl = document.createElement("div");
			codeEl.className = "source-line-content";
			codeEl.textContent = lineText || " ";

			lineEl.appendChild(numEl);
			lineEl.appendChild(codeEl);
			frag.appendChild(lineEl);
		});

		sourceContainer.appendChild(frag);
	}

	function scrollToLine(lineNum) {
		if (!lineNum) return;
		if (!state.showSource) {
			optShowSource.checked = true;
			state.showSource = true;
			renderSource();
		}
		const target = document.getElementById(`source-line-${lineNum}`);
		if (target) {
			target.scrollIntoView({ behavior: "smooth", block: "center" });
			target.classList.add("highlight-flash");
			setTimeout(() => target.classList.remove("highlight-flash"), 1600);
		}
	}

	function escapeHtml(str) {
		return str
			.replace(/&/g, "&amp;")
			.replace(/</g, "&lt;")
			.replace(/>/g, "&gt;")
			.replace(/"/g, "&quot;");
	}

	// Pre-load default sample
	directEditor.value = sampleInvalidDoc;
})();
