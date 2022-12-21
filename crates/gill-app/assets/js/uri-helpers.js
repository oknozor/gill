const navigation = (branch) => {
    let currentBranch = encodeURIComponent(branch);
    let {proto, host, user, repository, blobOrTree, currentPath} = pathInfo();
    let parts = currentPath.split("/");
    let linkElements = []
    linkElements.push(`<a href="${proto}//${host}/${user}/${repository}/tree/${currentBranch}">${repository}</a>`);
    let link = "";
    for (let i = 0; i < parts.length; i++) {
        let pathPart = parts[i];
        link = link + "/" + pathPart;
        let part = decodeURIComponent(pathPart);
        if (blobOrTree === "blob" && pathPart === parts[parts.length - 1]) {
            linkElements.push(`<span class="font-bold">${part}</span>`)
        } else {
            linkElements.push(`<a href="${proto}//${host}/${user}/${repository}/tree/${currentBranch}${link}">${part}</a>`);
        }
    }

    let navigationLinks = document.getElementById("navigation");
    navigationLinks.innerHTML = linkElements.join(" / ")
}


const setBranchDropDownLink = (branch) => {
    let {proto, host, user, repository, blobOrTree, currentPath} = pathInfo();
    let branchUri = encodeURIComponent(branch);
    let href;
    if (!blobOrTree) {
        href = `${proto}//${host}/${user}/${repository}/tree/${branchUri}`
    } else {
        if (currentPath) {
            href = `${proto}//${host}/${user}/${repository}/${blobOrTree}/${branchUri}/${currentPath}`
        } else {
            href = `${proto}//${host}/${user}/${repository}/${blobOrTree}/${branchUri}`
        }

    }

    let link = document.getElementById(`branch-${branch}`);
    link.setAttribute("href", href);
}

const generateTreeLink = (path, treeOrBLob, currentBranch) => {
    let branch = encodeURIComponent(currentBranch);
    let {proto, host, user, repository, currentPath} = pathInfo();
    let link = document.getElementById(path);
    let href;

    if (currentPath) {
        href = `${proto}//${host}/${user}/${repository}/${treeOrBLob}/${branch}/${currentPath}/${path}`
    } else {
        href = `${proto}//${host}/${user}/${repository}/${treeOrBLob}/${branch}/${path}`
    }

    link.setAttribute("href", href);
}

const pathInfo = () => {
    let proto = window.location.protocol;
    let host = window.location.host;
    let parts = window.location.pathname.split("/");
    parts.shift();

    let user = parts.shift();
    let repository = parts.shift();
    let blobOrTree = parts.shift();
    let _currentBranch = parts.shift();
    let currentPath = parts.join("/");

    return {
        proto,
        host,
        user,
        repository,
        blobOrTree,
        currentPath,
    }
}