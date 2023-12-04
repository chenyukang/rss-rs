<script>
    import { onMount } from "svelte";
    const jq = window.$;

    let file = "";
    let content = "";
    let show_status = false;
    let show_rsslink = false;
    let rsslink = "";
    let publish_time = "";
    let source = "";
    let rss_query_type = "unread";

    $: {
        fetchRss();
    }

    function fetchPage(url, query_type = "") {
        show_status = true;
        show_rsslink = false;
        let data = {
            path: url,
            query_type: query_type,
        };
        jq.ajax({
            url: "/api/page",
            data: data,
            type: "GET",
            datatype: "json",
            contentType: "Application/json",
            statusCode: {
                500: function () {
                    window.location.href = "/read";
                },
            },
            success: function (response) {
                show_status = false;
                file = response[0];
                content = response[1];
                rsslink = response[2];
                publish_time = response[3];
                source = response[4];
                if (file != "NoPage") {
                    jq("#fileName").text(file);
                    jq("#fileName").prop("hidden", false);
                    jq("#pageNavBar").prop("hidden", false);
                    jq("#page-content").html(content);
                    jq("#page-content").prop("hidden", false);
                    if (rsslink != undefined && rsslink != "") {
                        jq("#rsslink").prop("hidden", false);
                        show_rsslink = true;
                    }
                    setPageDefault();
                } else {
                    jq("#page-content").html("<h3>No Page</h3>");
                    jq("#fileName").text(url);
                }
            },
            error: function (err) {
                show_status = false;
                return err;
            },
        });
    }

    function hookInit() {
        jq(".pageContent")
            .off("click")
            .on("click", "a", function (e) {
                e.preventDefault();
                let url = e.target.href;
                if (!url) {
                    return false;
                }
                let link = e.target.getAttribute("id");
                if (link != null) {
                    fetchPage(link, "rss");
                } else {
                    window.open(url, "_blank");
                }
            });
    }

    function setPageDefault() {
        jq("#backBtn").prop("hidden", false);
        jq("#markBtn").prop("hidden", true);
        hookInit();
    }

    function markRead() {
        show_status = true;
        jq.ajax({
            url: "/api/rss_mark?index=0",
            type: "POST",
            data: "",
            datatype: "json",
            contentType: "Application/json",
            statusCode: {
                500: function () {
                    window.location.href = "/read";
                },
            },
            success: function (response) {
                show_status = false;
                if (response == "ok") {
                    jq("#markBtn").prop("hidden", false);
                    fetchRss();
                }
            },
            error: function (err) {
                show_status = false;
                return err;
            },
        });
    }

    function markRemove() {
        show_status = true;
        // get the value of rsslink
        let rsslink = jq("#rsslink").attr("href");
        console.log(rsslink);
        let data = {
            link: rsslink,
        };
        let url = "/api/rss_remove";
        jq.ajax({
            url: url,
            type: "POST",
            data: JSON.stringify(data),
            datatype: "json",
            contentType: "Application/json",
            statusCode: {
                500: function () {
                    window.location.href = "/read";
                },
            },
            success: function (response) {
                show_status = false;
                if (response == "ok") {
                    jq("#markRemove").prop("hidden", true);
                }
            },
            error: function (err) {
                show_status = false;
                return err;
            },
        });
    }

    function fetchRss() {
        show_status = true;
        show_rsslink = false;
        rss_query_type = localStorage.getItem("rss_query_type") || "unread";
        let data = {
            query_type: rss_query_type,
            limit: 100,
        };
        jq.ajax({
            url: "/api/rss",
            type: "GET",
            data: data,
            datatype: "json",
            contentType: "Application/json",
            statusCode: {
                500: function () {
                    window.location.href = "/read";
                },
            },
            success: function (response) {
                show_status = false;
                if (response != "no-page") {
                    jq("#page-content").html(response);
                    jq("#page-content").prop("hidden", false);
                    jq("#fileName").prop("hidden", true);
                    jq("#backBtn").prop("hidden", true);
                    jq("#markBtn").prop("hidden", false);
                    jq("#pageNavBar").prop("hidden", true);
                    jq("#rssread").prop("checked", rss_query_type == "all");
                } else {
                    jq("#page-content").html(
                        "<h3>No Page</h3>" + " " + local_date,
                    );
                }
            },
            error: function (err) {
                show_status = false;
                return err;
            },
        });
    }

    function rssRead() {
        rss_query_type = jq(this).prop("checked") === true ? "all" : "unread";
        localStorage.setItem("rss_query_type", rss_query_type);
        fetchRss();
    }

    onMount(async () => {
        setPageDefault();
    });
</script>

<main>
    <div class="container">
        <div class="tab-content">
            <div class="row sticky-top" style="margin-top: 20px; border: 0;">
                <div class="col-md-2" />
                <div class="col-md-8 text-right" id="pageNavBarRss">
                    <button
                        type="button"
                        class="btn btn-info"
                        style="float: left"
                        id="backBtn"
                        hidden="true"
                        on:click={fetchRss}>Back</button
                    >

                    <button
                        type="button"
                        class="btn btn-info"
                        style="float: left"
                        id="markBtn"
                        hidden="true"
                        on:click={markRead}>Mark</button
                    >

                    {#if !show_rsslink}
                        <label class="switch" style="float: right">
                            <input
                                id="rssread"
                                type="checkbox"
                                on:click={rssRead}
                            />
                            <span class="slider round"></span>
                        </label>
                    {:else}
                        <button
                            type="button"
                            class="btn btn-info"
                            style="float: right"
                            id="markRemove"
                            on:click={markRemove}>Unsubscribe</button
                        >
                    {/if}
                </div>
            </div>

            {#if show_status}
                <div class="row">
                    <div class="col-md-2" />
                    <div
                        class="col-md-8"
                        id="status-sp"
                        style="margin-top: 20px;"
                    >
                        <div class="text-center">
                            <div
                                class="spinner-border text-success"
                                role="status"
                            >
                                <span class="sr-only" />
                            </div>
                        </div>
                    </div>
                </div>
            {/if}

            <div class="row">
                <div class="col-md-2" />
                <div class="col-md-8">
                    <div class="text-center" style="margin-top: 20px;">
                        <h4>
                            <span
                                class="badge badge-secondary"
                                style="white-space: pre-line;"
                                hidden="true"
                                id="fileName"
                            />
                        </h4>
                    </div>
                </div>
                <div class="col-md-2" />
            </div>

            {#if show_rsslink}
                <div class="row">
                    <div class="col-md-2" />
                    <div class="col-md-8" style="text-align: center;">
                        <a href={rsslink} id="rsslink" target="_blank"
                            >{publish_time.split(" ")[0]} 👻 {new URL(rsslink)}
                        </a>
                    </div>
                </div>
            {/if}

            <div class="row">
                <div class="col-md-2" />
                <div class="col-md-8">
                    <div class="pageContent" hidden="true" id="page-content" />
                </div>
            </div>
        </div>
    </div>
</main>
