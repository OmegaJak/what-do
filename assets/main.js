setInterval(createSortable, 500);

function createSortable() {
	var sortableList = document.getElementById("sortableList");
	if (sortableList && !Sortable.get(sortableList)) {
		Sortable.create(sortableList, {
			animation: 100,
			store: {
				set: function (sortable) {
					var sortableList = document.getElementById("sortableList");
					var sortingOutput = document.getElementById("sortingOutput");
					if (sortableList && sortingOutput) {
						var arr = [];
						for (var child of sortableList.children) {
							var id = child.getAttribute("option-id");
							if (id) {
								arr.push(id);
							}
						}

						sortingOutput.value = arr.join('\n');
					} else {
						console.error("Failed to find sortableList and/or sortingOutput");
					}
				}
			}
		});
	}
}