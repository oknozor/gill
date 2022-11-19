const dirLink = path => {
    let href = window.location.href + "/" + path
    let link = document.getElementById(path);
    link.setAttribute("href", href);
}

const blobLink = path => {
    let location = window.location.pathname;
    let href = location.replace("/tree", "/blob") + "/" + path;
    let link = document.getElementById(path);
    link.setAttribute("href", href);
}
