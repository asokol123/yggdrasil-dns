package main

import (
	"bytes"
	"crypto/ecdsa"
	"crypto/rand"
	"crypto/sha256"
	"crypto/x509"
	"encoding/hex"
	"encoding/json"
	"encoding/pem"
	"fmt"
	"github.com/spf13/cobra"
	"io"
	"net/http"
	"os"
	"strconv"
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
	Signature string `json:"signature"`
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

func check(e error) {
	if e != nil {
		panic(e)
	}
}

func addProofOfWork(req HasNonce, difficulty int) {
	h := sha256.New()
	for !strings.HasPrefix(hex.EncodeToString(h.Sum(nil)), strings.Repeat("0", difficulty)) {
		req.IncNonce()
		h.Reset()

		data, err := json.Marshal(req)
		check(err)
		h.Write(data)
	}
}

func request(method string, url string, body interface{}, timeout time.Duration) {
	encodedBody, err := json.Marshal(body)
	check(err)

	client := http.Client{Timeout: timeout}
	req, err := http.NewRequest(method, url, bytes.NewReader(encodedBody))
	check(err)

	resp, err := client.Do(req)
	check(err)
	fmt.Println("Status: ", resp.Status)

	respBody, err := io.ReadAll(resp.Body)
	check(err)
	fmt.Println("Response: ", string(respBody))
}

func main() {
	var (
		rr                RegisterRequest
		ssr               SetSiteRequest
		gsr               GetSiteRequest
		endpoint          string
		timeout           time.Duration
		difficulty        int
		signatureFilename string
	)

	var rootCmd = &cobra.Command{}
	rootCmd.PersistentFlags().StringVarP(&endpoint, "endpoint", "e", "", "request endpoint")
	rootCmd.PersistentFlags().DurationVarP(&timeout, "timeout", "t", DefaultTimeout, "request timeout")
	rootCmd.PersistentFlags().IntVarP(&difficulty, "pow-zeros", "z", DefaultDifficulty, "required number of zeros in pow")
	rootCmd.MarkPersistentFlagRequired("endpoint")

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
			request("POST", "http://"+endpoint+"/register", rr, timeout)
		},
	}
	registerCmd.Flags().StringVarP(&rr.Name, "name", "n", "", "user to register")
	registerCmd.MarkFlagRequired("name")

	var setSiteCmd = &cobra.Command{
		Use:   "set_site --site [site] --address [address] --expires [expires] --owner [owner] -sf [signature_filename]",
		Short: "Create a new site or update the existing one",
		Run: func(cmd *cobra.Command, args []string) {
			ssr.Timestamp = int(time.Now().Unix())
			data, err := os.ReadFile(signatureFilename)
			check(err)

			block, _ := pem.Decode(data)
			if block == nil {
				panic("Invalid signature")
			}
			privateKey, err := x509.ParseECPrivateKey(block.Bytes)
			check(err)
			msg := sha256.Sum256([]byte(ssr.Owner + ssr.Site + strconv.Itoa(ssr.Timestamp)))
			signature, err := ecdsa.SignASN1(rand.Reader, privateKey, msg[:])
			ssr.Signature = hex.EncodeToString(signature)

			addProofOfWork(&ssr, difficulty)
			request("POST", "http://"+endpoint+"/set_site", ssr, timeout)
		},
	}
	setSiteCmd.Flags().StringVar(&ssr.Site, "site", "", "site to find")
	setSiteCmd.Flags().StringVarP(&ssr.Address, "address", "a", "", "site's ip")
	setSiteCmd.Flags().IntVarP(&ssr.Expires, "expires", "E", 0, "site's expiration time")
	setSiteCmd.Flags().StringVarP(&ssr.Owner, "owner", "s", "", "site to find")
	setSiteCmd.Flags().StringVarP(&signatureFilename, "signature-filename", "f", "", "file with a signature")
	for _, flag := range []string{"site", "address", "expires", "owner", "signature filename"} {
		setSiteCmd.MarkFlagRequired(flag)
	}

	var getSiteCmd = &cobra.Command{
		Use:   "get_site --site [site]",
		Short: "Get a site ip by it's name",
		Run: func(cmd *cobra.Command, args []string) {
			gsr.Timestamp = int(time.Now().Unix())
			addProofOfWork(&gsr, difficulty)
			request("GET", "http://"+endpoint+"/get_site", gsr, timeout)
		},
	}
	getSiteCmd.Flags().StringVarP(&gsr.Site, "site", "s", "", "site to find")
	getSiteCmd.MarkFlagRequired("site")

	rootCmd.AddCommand(registerCmd)
	rootCmd.AddCommand(setSiteCmd)
	rootCmd.AddCommand(getSiteCmd)
	rootCmd.Execute()
}
