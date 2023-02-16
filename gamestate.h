struct ss{
  char n0[20],n1[20],n2[20],n3[20];
  char p02,p13;
  char state;
  char bid;
  char h0[13],h1[13],h2[13],h3[13];
  char k[5];
  char ip[4];
  char ws[52];
  char wp;
};
struct state{
  char mode; // 0 bid 1 kitty / trump 2 play 3 done
  char turn;
  char trmp; // rgbyn
};
