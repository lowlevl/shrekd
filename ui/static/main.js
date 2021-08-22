$(() => {

    $("#file-upload-input").change(function (event) {
        $("#file-upload-output-outer").hide();
        let file = $(this).prop("files")[0];

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

            $("#file-upload-output").text(req.responseJSON.message);
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
        $("#paste-creation-output-outer").hide();
        let snippet = $("#paste-creation-input").val();

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

            $("#paste-creation-output").text(req.responseJSON.message);
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

});
