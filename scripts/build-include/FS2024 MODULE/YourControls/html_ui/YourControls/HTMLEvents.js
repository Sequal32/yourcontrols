class YourControlsHTMLEvents {
	constructor(buttonCallback, inputCallback) {
		this.bindedInputs = {};
		this.buttonCallback = buttonCallback;
		this.inputCallback = inputCallback;
		this.documentListenerLoop = null;
		this.documentMouseListener = null;
	}

	bindEvents() {
		this.stopMouseUpListener();

		const mouseUpListener = (e) => {
			if (e.YC) {
				return;
			}

			if (e.button !== 0) { // Left-click only
				return;
			}

			let currentWorking = e.target;

			while (currentWorking.id == "" && currentWorking != null) {
				currentWorking = currentWorking.parentNode;
			}

			if (!currentWorking) {
				return;
			}

			this.buttonCallback(currentWorking.id);
		};
		document.addEventListener("mouseup", mouseUpListener);
		this.documentMouseListener = mouseUpListener;
	}

	stopMouseUpListener() {
		if (this.documentMouseListener === null) {
			return;
		}
		document.removeEventListener("mouseup", this.documentMouseListener);
		this.documentMouseListener = null;
	}

	startDocumentListener() {
		this.stopDocumentListener();

		const addElement = this.addElement.bind(this);

		this.documentListenerLoop = setInterval(() => {
			document.querySelectorAll("*").forEach((element) => {
				addElement(element);
			});
		}, 400);
	}

	stopDocumentListener() {
		if (this.documentListenerLoop === null) {
			return;
		}
		clearInterval(this.documentListenerLoop);
		this.documentListenerLoop = null;
	}

	addElement(element) {
		this.addButton(element);
		this.addInput(element);
	}

	getHash(string) {
		let hash = 0;
		for (let i = 0; i < string.length; i++) {
			const chr = string.charCodeAt(i);
			hash = ((hash << 5) - hash + chr) | 0;
		}
		return hash >>> 0;
	}

	getPositionOfElementInParent(element) {
		const parent = element.parentElement;
		if (!parent) return 0;

		let position = 1;
		let sibling = element.previousElementSibling;
		while (sibling) {
			position++;
			sibling = sibling.previousElementSibling;
		}
		return position;
	}

	countParents(element) {
		let count = 0;
		let workingElement = element.parentElement;

		while (workingElement) {
			count++;
			workingElement = workingElement.parentElement;
		}
		return count;
	}

	getAttributesAsOneString(element) {
		let longString = "";

		if (element.hasAttributes()) {
			let attrs = Array.from(element.attributes);
			attrs.sort((a, b) => a.name.localeCompare(b.name));

			for (let attr of attrs) {
				longString += attr.name + "#" + attr.value;
			}
		}
		return longString;
	}

	generateHTMLHash(element) {
		const baseString = element.tagName.toLowerCase() + "|" + this.getAttributesAsOneString(element);
		let hash = this.getHash(baseString);

		const depth = this.countParents(element);
		const pos = this.getPositionOfElementInParent(element);

		hash = (hash * 1000000 + depth * 1000 + pos) >>> 0;

		return hash >>> 0;
	}

	getIdCorrected(id) {
		while (document.getElementById(id) != null) {
			id += 1;
		}

		return id;
	}

	addButton(element) {
		if (element.id !== "") {
			return;
		}

		const generated = this.generateHTMLHash(element);
		const id = this.getIdCorrected(generated);
		element.id = id;
	}

	addInput(element) {
		if (!(element instanceof HTMLInputElement) || this.bindedInputs[element.id]) {
			return;
		}
		this.bindedInputs[element.id] = true;

		let cacheValue = null;

		const inputHandler = (e) => {
			if (e && e.YC) {
				return;
			}
			if (cacheValue === element.value) {
				return;
			}
			cacheValue = element.value;
			this.inputCallback(element.id, element.value); // Send value
		};
		element.addEventListener("input", inputHandler);
	}

	clear() {
		this.bindedInputs = {};
		this.stopDocumentListener();
		this.stopMouseUpListener();
	}
}