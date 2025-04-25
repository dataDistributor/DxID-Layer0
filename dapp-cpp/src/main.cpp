#include <iostream>
#include <string>
#include <curl/curl.h>

// Callback function to collect HTTP response data.
static size_t WriteCallback(void* contents, size_t size, size_t nmemb, void* userp) {
    std::string* s = static_cast<std::string*>(userp);
    size_t totalSize = size * nmemb;
    s->append(static_cast<char*>(contents), totalSize);
    return totalSize;
}

// Function to perform HTTP GET request.
std::string http_get(const std::string &url) {
    CURL *curl = curl_easy_init();
    std::string response;
    if (curl) {
        curl_easy_setopt(curl, CURLOPT_URL, url.c_str());
        curl_easy_setopt(curl, CURLOPT_FOLLOWLOCATION, 1L);
        curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, WriteCallback);
        curl_easy_setopt(curl, CURLOPT_WRITEDATA, &response);
        CURLcode res = curl_easy_perform(curl);
        if (res != CURLE_OK) {
            std::cerr << "GET error: " << curl_easy_strerror(res) << std::endl;
        }
        curl_easy_cleanup(curl);
    }
    return response;
}

// Function to perform HTTP POST request with JSON payload.
std::string http_post(const std::string &url, const std::string &json_data) {
    CURL *curl = curl_easy_init();
    std::string response;
    if (curl) {
        struct curl_slist *headers = nullptr;
        headers = curl_slist_append(headers, "Content-Type: application/json");
        curl_easy_setopt(curl, CURLOPT_URL, url.c_str());
        curl_easy_setopt(curl, CURLOPT_HTTPHEADER, headers);
        curl_easy_setopt(curl, CURLOPT_POSTFIELDS, json_data.c_str());
        curl_easy_setopt(curl, CURLOPT_WRITEFUNCTION, WriteCallback);
        curl_easy_setopt(curl, CURLOPT_WRITEDATA, &response);
        CURLcode res = curl_easy_perform(curl);
        if (res != CURLE_OK) {
            std::cerr << "POST error: " << curl_easy_strerror(res) << std::endl;
        }
        curl_slist_free_all(headers);
        curl_easy_cleanup(curl);
    }
    return response;
}

int main() {
    const std::string base_url = "http://127.0.0.1:3030";

    // POST: Add a new block.
    std::string post_url = base_url + "/add_block";
    std::string json_payload = R"({"data": "Block added from C++ dApp"})";
    std::cout << "Sending POST request to add a new block..." << std::endl;
    std::string post_response = http_post(post_url, json_payload);
    std::cout << "POST Response: " << post_response << std::endl;

    // GET: Retrieve the blockchain.
    std::string get_url = base_url + "/chain";
    std::cout << "Sending GET request to fetch blockchain data..." << std::endl;
    std::string get_response = http_get(get_url);
    std::cout << "Blockchain Data:" << std::endl;
    std::cout << get_response << std::endl;

    return 0;
}
