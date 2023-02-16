#include"gamestate.h"
#include<stdio.h>
#include<stdlib.h>
#include<time.h>
char encode_state(struct state*s){
  return s->mode&3|s->turn<<2&4|s->trmp<<3&24;
}
struct state decode_state(char s){
  struct state ss={s&3,s&4,s&24};
  return ss;
}
//card: 0rssvvvv, 1 stored as 15; rgby
char ptcd(char c){
  if(c&64)return 20;
  switch(c&15){
  case 15:
	return 15;
  case 14:
  case 10:
	return 10;
  case 5:
	return 5;
  default:
	return 0;
  }
}
void initcd(struct ss*s){
  for(int c=0;c<4;c++)
	for(int v=0;v<14;v++)
	  s->h0[c*14+v]=c<<4|(v+2);
  s->k[4]=64;
}
void shfcd(struct ss*s){
  for(int i=0;i<56;i++){
    int r=rand()%(57-i)+i;
	char tmp=s->h0[i];
	s->h0[i]=s->h0[r];
	s->h0[r]=tmp;
  }
}
void printcd(char crd){
  switch(crd&112){
  case 64:
	printf("rook (20)");
	return;
  case 0:
	printf("r ");
	break;
  case 16:
	printf("g ");
	break;
  case 32:
	printf("b ");
	break;
  case 48:
	printf("y ");
	break;
  default:printf("INVALID ");
  }
  if((crd&15)==15)
	printf("1");
  else
	printf("%d",crd&15);
  char p=ptcd(crd);
  if(p)
	printf(" (%d)",p);
}
void printcds(struct ss*s){
  for(int h=0;h<4;h++){
	printf("player%d: ",h);
	for(int c=0;c<13;c++){
	  char crd=s->h0[h*13+c];
	  if(!crd)continue;
	  if(c)putchar(',');
	  printcd(crd);
	}
	putchar('\n');
  }
  printf("kitty: ");
  for(int c=0;c<5;c++){
	if(c)putchar(',');
	printcd(s->k[c]);
  }
  putchar('\n');
}
int main(){
  srand(time(0));
  struct ss s;
  initcd(&s);
  shfcd(&s);
  printcds(&s);
}
