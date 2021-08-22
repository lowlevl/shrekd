$(() => {

    $("#file-upload-input").change(function (event) {
        let file = $(this).prop("files")[0];

        $("#file-upload-output-outer").hide();

        $.ajax({
            url: `/${encodeURIComponent(file.name)}`,
            type: "PUT",
            data: file,
            processData: false,
            contentType: false,
        })
        .catch((req, _, error) => {
            console.error(`File upload failed because of ${req.status} ${error} !`);
            console.error(req, error);

            if (req.status == 0) {
                $("#file-upload-output").text("A network error occured or the size limit has been reached");
            } else {
                $("#file-upload-output").text(req.responseJSON.message);
            }
            $("#file-upload-output").removeClass("status-ok");
            $("#file-upload-output").addClass("status-ko");
            $("#file-upload-output-outer").show();
        })
        .then((data, _, req) => {
            console.log(`File upload succeeded with a status of ${req.status} !`);

            $("#file-upload-output").text(data);
            $("#file-upload-output").removeClass("status-ko");
            $("#file-upload-output").addClass("status-ok");
            $("#file-upload-output-outer").show();
        });
    });

    $("#paste-creation-button").click(function (event) {
        let snippet = $("#paste-creation-input").val();

        if (!snippet) {
            event.preventDefault();
            return;
        }

        $("#paste-creation-output-outer").hide();

        $.ajax({
            url: "/paste",
            type: "POST",
            data: snippet,
            processData: false,
            contentType: false,
        })
        .catch((req, _, error) => {
            console.error(`Paste creation failed because of ${req.status} ${error} !`);
            console.error(req, error);

            if (req.status == 0) {
                $("#paste-creation-output").text("A network error occured");
            } else {
                $("#paste-creation-output").text(req.responseJSON.message);
            }
            $("#paste-creation-output").removeClass("status-ok");
            $("#paste-creation-output").addClass("status-ko");
            $("#paste-creation-output-outer").show();
        })
        .then((data, _, req) => {
            console.log(`Paste creation succeeded with a status of ${req.status} !`);

            $("#paste-creation-output").text(data);
            $("#paste-creation-output").removeClass("status-ko");
            $("#paste-creation-output").addClass("status-ok");
            $("#paste-creation-output-outer").show();
        });
    });

    $("#redirect-creation-button").click(function (event) {
        let link = $("#redirect-creation-input").val();

        if (!link) {
            event.preventDefault();
            return;
        }

        $("#paste-creation-output-outer").hide();

        $.ajax({
            url: "/url",
            type: "POST",
            data: link,
            processData: false,
            contentType: false,
        })
        .catch((req, _, error) => {
            console.error(`Redirect creation failed because of ${req.status} ${error} !`);
            console.error(req, error);

            if (req.status == 0) {
                $("#redirect-creation-output").text("A network error occured");
            } else {
                $("#redirect-creation-output").text(req.responseJSON.message);
            }
            $("#redirect-creation-output").removeClass("status-ok");
            $("#redirect-creation-output").addClass("status-ko");
            $("#redirect-creation-output-outer").show();
        })
        .then((data, _, req) => {
            console.log(`Redirect creation succeeded with a status of ${req.status} !`);

            $("#redirect-creation-output").text(data);
            $("#redirect-creation-output").removeClass("status-ko");
            $("#redirect-creation-output").addClass("status-ok");
            $("#redirect-creation-output-outer").show();
        });
    });

});
