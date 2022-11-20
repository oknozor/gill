const navigation = () => {
    let proto = window.location.protocol;
    let host = window.location.host;
    let parts = window.location.pathname.split("/");
    parts.shift(); // Remove empty parts
    let user = parts.shift();
    let repo = parts.shift();
    parts.shift(); // Remove `blob` or `tree`
    let currentBranch = parts.shift();
    let currentPage = parts.pop();
    let link = "";
    let linkElements = []
    linkElements.push(`<a href="${proto}//${host}/${user}/${repo}/tree/${currentBranch}">${repo}</a>`);
    for (let part of parts) {
        link = link + "/" + part;
        let label = decodeURIComponent(part);
        linkElements.push(`<a href="${proto}//${host}/${user}/${repo}/tree/${currentBranch}${link}">${label}</a>`);
    }

    if (currentPage) {
        linkElements.push(`<span class="font-bold">${currentPage}</span>`);
    }

    let navigationLinks = document.getElementById("navigation");
    navigationLinks.innerHTML = linkElements.join("/")
}

const changeBranch = (branch) => {
    let proto = window.location.protocol;
    let host = window.location.host;
    let parts = window.location.pathname.split("/");
    parts.shift();
    let user = parts.shift();
    let repo = parts.shift();
    let blobOrTree = parts.shift();
    // Current branch
    parts.shift();
    let tree = parts.join("/");
    let branchPath = encodeURIComponent(branch.value);
    window.location.href = `${proto}//${host}/${user}/${repo}/${blobOrTree}/${branchPath}/${tree}`
}

const setCurrentBranchSelected = (branch) => {
    let select = document.getElementById("branches");
    select.value = branch;
}