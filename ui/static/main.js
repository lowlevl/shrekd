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

            $("#file-upload-output").text(error);
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

});
