package main

import (
	"bytes"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"github.com/spf13/cobra"
	"io"
	"log"
	"net/http"
	"os"
	"strings"
	"time"
)

const (
	DefaultTimeout    = time.Duration(5) * time.Second
	DefaultDifficulty = 4
)

type HasNonce interface {
	IncNonce()
}

type RegisterRequest struct {
	Name      string `json:"name"`
	PubKey    string `json:"pubkey"`
	Timestamp int    `json:"timestamp"`
	Nonce     int    `json:"nonce"`
}

func (r *RegisterRequest) IncNonce() {
	r.Nonce += 1
}

type SetSiteRequest struct {
	Site      string `json:"site"`
	Address   string `json:"address"`
	Expires   int    `json:"expires"`
	Owner     string `json:"owner"`
	Timestamp int    `json:"timestamp"`
	Nonce     int    `json:"nonce"`
}

func (r *SetSiteRequest) IncNonce() {
	r.Nonce += 1
}

type GetSiteRequest struct {
	Site      string `json:"site"`
	Timestamp int    `json:"timestamp"`
	Nonce     int    `json:"nonce"`
}

func (r *GetSiteRequest) IncNonce() {
	r.Nonce += 1
}

func addProofOfWork(req HasNonce, difficulty int) {
	h := sha256.New()
	for !strings.HasPrefix(hex.EncodeToString(h.Sum(nil)), strings.Repeat("0", difficulty)) {
		req.IncNonce()
		h.Reset()

		data, err := json.Marshal(req)
		if err != nil {
			panic(err)
		}
		h.Write(data)
	}
}

func request(method string, url string, body interface{}, timeout time.Duration) {
	encodedBody, err := json.Marshal(body)
	if err != nil {
		panic(err)
	}

	client := http.Client{Timeout: timeout}
	req, err := http.NewRequest(method, url, bytes.NewReader(encodedBody))
	if err != nil {
		panic(err)
	}

	resp, err := client.Do(req)
	if err != nil {
		panic(err)
	}
	fmt.Println("Status: ", resp.Status)

	respBody, err := io.ReadAll(resp.Body)
	if err != nil {
		log.Fatal(err)
	}
	fmt.Println("Response: ", string(respBody))
}

func main() {
	var (
		rr RegisterRequest
		//ssr        SetSiteRequest
		gsr        GetSiteRequest
		endpoint   string
		timeout    time.Duration
		difficulty int
	)

	var rootCmd = &cobra.Command{}
	rootCmd.PersistentFlags().StringVarP(&endpoint, "endpoint", "e", "", "request endpoint")
	rootCmd.PersistentFlags().DurationVarP(&timeout, "timeout", "t", DefaultTimeout, "request timeout")
	rootCmd.PersistentFlags().IntVarP(&difficulty, "pow-zeros", "z", DefaultDifficulty, "required number of zeros in pow")

	var registerCmd = &cobra.Command{
		Use:   "register --name [name]",
		Short: "Register a new user",
		Run: func(cmd *cobra.Command, args []string) {
			rr.PubKey = os.Getenv("PUBLIC_KEY")
			if rr.PubKey == "" {
				panic("empty PUBLIC_KEY")
			}

			rr.Timestamp = int(time.Now().Unix())
			addProofOfWork(&rr, difficulty)
			request("POST" /*"http://"+*/, endpoint+"/register", rr, timeout)
		},
	}
	registerCmd.Flags().StringVarP(&rr.Name, "name", "n", "", "user to register")

	var getSiteCmd = &cobra.Command{
		Use:   "get_site --site [site]",
		Short: "Get a site ip by it's name",
		Run: func(cmd *cobra.Command, args []string) {
			gsr.Timestamp = int(time.Now().Unix())
			addProofOfWork(&gsr, difficulty)
			request("GET" /*"http://"+*/, endpoint+"/get_site", gsr, timeout)
		},
	}
	getSiteCmd.Flags().StringVarP(&gsr.Site, "site", "s", "", "site to find")

	rootCmd.AddCommand(registerCmd)
	rootCmd.AddCommand(getSiteCmd)
	rootCmd.Execute()
}
